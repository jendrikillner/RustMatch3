use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use winapi::shared::dxgi::*;
use winapi::shared::dxgi1_2::*;
use winapi::shared::dxgiformat::*;
use winapi::shared::dxgitype::*;
use winapi::shared::minwindef::{HINSTANCE, LPARAM, LRESULT, UINT, WPARAM};
use winapi::shared::ntdef::HRESULT;
use winapi::shared::ntdef::LPCWSTR;
use winapi::shared::windef::{HBRUSH, HICON, HMENU, HWND};
use winapi::um::d3d11::*;
use winapi::um::d3d11_1::*;
use winapi::um::d3dcommon::*;
use winapi::um::winuser::*;
use winapi::Interface;

use std_time_ext::*;

struct WindowCreatedData {
    hwnd: HWND,
}

unsafe impl std::marker::Send for WindowCreatedData {}

enum WindowMessages {
    WindowCreated(WindowCreatedData),
    WindowClosed,
}

struct Window {
    message_receiver: std::sync::mpsc::Receiver<WindowMessages>,
    hwnd: HWND,
}

struct WindowThreadState {
    message_sender: std::sync::mpsc::Sender<WindowMessages>,
}

unsafe extern "system" fn window_proc(
    h_wnd: HWND,
    msg: UINT,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if msg == WM_CREATE {
        // retrieve the message struct that contains the creation parameters
        let create_struct = l_param as *mut winapi::um::winuser::CREATESTRUCTW;

        // retrieve the rust window state
        let window_state_ptr =
            create_struct.as_ref().unwrap().lpCreateParams as *mut WindowThreadState;
        let window_state: &mut WindowThreadState = window_state_ptr.as_mut().unwrap();

        // the state we can store inside the user data parameter of the window
        SetWindowLongPtrW(h_wnd, GWLP_USERDATA, window_state_ptr as isize);

        window_state
            .message_sender
            .send(WindowMessages::WindowCreated(WindowCreatedData {
                hwnd: h_wnd,
            }))
            .unwrap();
    }

    if msg == WM_DESTROY {
        let window_state_ptr = GetWindowLongPtrW(h_wnd, GWLP_USERDATA) as *mut WindowThreadState;
        let window_state: &mut WindowThreadState = window_state_ptr.as_mut().unwrap();

        window_state
            .message_sender
            .send(WindowMessages::WindowClosed)
            .unwrap();

        PostQuitMessage(0);
    }

    DefWindowProcW(h_wnd, msg, w_param, l_param)
}

fn create_window() -> Result<Window, ()> {
    let (channel_sender, channel_receiver) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        let mut window_state = WindowThreadState {
            message_sender: channel_sender,
        };

        unsafe {
            let mut window_class_name: Vec<u16> =
                OsStr::new("Match3WindowClass").encode_wide().collect();

            window_class_name.push(0);

            let window_class = WNDCLASSW {
                style: 0,
                lpfnWndProc: Some(window_proc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: 0 as HINSTANCE,
                hIcon: 0 as HICON,
                hCursor: 0 as HICON,
                hbrBackground: 16 as HBRUSH,
                lpszMenuName: 0 as LPCWSTR,
                lpszClassName: window_class_name.as_ptr(),
            };

            let error_code = RegisterClassW(&window_class);

            assert!(error_code != 0, "failed to register the window class");

            let h_wnd_window = CreateWindowExW(
                0,
                window_class_name.as_ptr(),
                0 as LPCWSTR,
                WS_OVERLAPPED | WS_MINIMIZEBOX | WS_SYSMENU,
                0,
                0,
                400,
                400,
                0 as HWND,
                0 as HMENU,
                0 as HINSTANCE,
                &mut window_state as *mut WindowThreadState as *mut winapi::ctypes::c_void, // pass a mutable pointer to the window
            );

            assert!(h_wnd_window != (0 as HWND), "failed to open the window");

            ShowWindow(h_wnd_window, SW_SHOW);

            let mut msg: MSG = std::mem::zeroed();

            // process messages
            loop {
                if PeekMessageA(&mut msg, h_wnd_window, 0, 0, PM_REMOVE) > 0 {
                    TranslateMessage(&msg);
                    DispatchMessageA(&msg);

                    // once the window has been closed we can exit the message loop
                    if msg.message == WM_QUIT {
                        break;
                    }
                }
            }
        }
    });

    // wait for window created before returning
    if let WindowMessages::WindowCreated(x) = channel_receiver.recv().unwrap() {
        return Ok(Window {
            message_receiver: channel_receiver,
            hwnd: x.hwnd,
        });
    }

    Err(())
}

fn process_window_messages(window: &Window) -> Option<WindowMessages> {
    if let Ok(x) = window.message_receiver.try_recv() {
        return Some(x);
    }

    None
}

#[repr(C)]
struct Float3 {
    x: f32,
    y: f32,
    z: f32,
}

#[repr(C)]
struct Float2 {
    x: f32,
    y: f32,
}

#[repr(C)]
struct ScreenSpaceQuadData {
    color: Float3,
    padding: f32,
    scale: Float2,
    position: Float2,
}

pub struct MappedGpuData<'a> {
    data: &'a [u8],               // reference to slice of cpu accessible gpu memory
    buffer: &'a mut ID3D11Buffer, // reference to the d3d11 buffer the data comes from
}

fn map_gpu_buffer<'a>(
    buffer: &'a mut ID3D11Buffer,
    context: &ID3D11DeviceContext,
) -> MappedGpuData<'a> {
    let mut mapped_resource = D3D11_MAPPED_SUBRESOURCE {
        pData: std::ptr::null_mut(),
        RowPitch: 0,
        DepthPitch: 0,
    };

    // map the buffer
    let result: HRESULT = unsafe {
        context.Map(
            buffer as *mut ID3D11Buffer as *mut winapi::um::d3d11::ID3D11Resource,
            0,
            D3D11_MAP_WRITE_NO_OVERWRITE,
            0,
            &mut mapped_resource,
        )
    };

    assert!(result == winapi::shared::winerror::S_OK);

    MappedGpuData {
        data: unsafe {
            std::slice::from_raw_parts_mut(
                mapped_resource.pData as *mut u8,
                mapped_resource.RowPitch as usize,
            )
        },
        buffer,
    }
}

fn unmap_gpu_buffer(mapped_data: MappedGpuData, context: &ID3D11DeviceContext) {
    unsafe {
        context.Unmap(
            mapped_data.buffer as *mut ID3D11Buffer as *mut winapi::um::d3d11::ID3D11Resource,
            0,
        );
    }
}

pub struct LinearAllocatorState {
    used_bytes: usize,
}

pub struct LinearAllocator<'a> {
    gpu_data: MappedGpuData<'a>,

    state: LinearAllocatorState,
}

pub struct HeapAlloc<'a, T> {
    ptr: &'a mut T,
    first_constant_offset: u32,
    num_constants: u32,
}

fn round_up_to_multiple(number: usize, multiple: usize) -> usize {
    ((number + multiple - 1) / multiple) * multiple
}

impl<'a, T> HeapAlloc<'a, T> {
    pub fn new(
        x: T,
        gpu_data: &'a MappedGpuData,
        state: &mut LinearAllocatorState,
    ) -> HeapAlloc<'a, T> {
        let allocation_size: usize = round_up_to_multiple(std::mem::size_of::<T>(), 256);

        let data_slice = gpu_data.data;
        let start_offset_in_bytes = state.used_bytes;
        // let end_offset_in_byes    = allocator.used_bytes + allocation_size;

        let data_ptr =
            data_slice[state.used_bytes..(state.used_bytes + allocation_size)].as_ptr() as *mut T;

        state.used_bytes += allocation_size;

        unsafe {
            // write data into target destination
            std::ptr::write(data_ptr, x);

            HeapAlloc {
                ptr: data_ptr.as_mut().unwrap(),
                first_constant_offset: (start_offset_in_bytes / 16) as u32,
                num_constants: (allocation_size / 16) as u32,
            }
        }
    }
}

impl<T> std::ops::Deref for HeapAlloc<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.ptr
    }
}

impl<T> std::ops::DerefMut for HeapAlloc<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.ptr
    }
}

struct GpuBuffer {
    native_buffer: *mut ID3D11Buffer,
}

fn create_constant_buffer(device: &ID3D11Device, size_in_bytes: u32) -> GpuBuffer {
    let mut per_draw_buffer: *mut ID3D11Buffer = std::ptr::null_mut();

    let buffer_desc = D3D11_BUFFER_DESC {
        ByteWidth: size_in_bytes,
        Usage: D3D11_USAGE_DYNAMIC,
        BindFlags: D3D11_BIND_CONSTANT_BUFFER,
        CPUAccessFlags: D3D11_CPU_ACCESS_WRITE,
        MiscFlags: 0,
        StructureByteStride: 0,
    };

    let error =
        unsafe { device.CreateBuffer(&buffer_desc, std::ptr::null(), &mut per_draw_buffer) };

    assert!(error == winapi::shared::winerror::S_OK);

    GpuBuffer {
        native_buffer: per_draw_buffer,
    }
}

struct CpuRenderFrameData {
    frame_constant_buffer: GpuBuffer,
}

struct GraphicsDeviceLayer {
    device: *mut ID3D11Device,
    immediate_context: *mut ID3D11DeviceContext,
    swapchain: *mut IDXGISwapChain1,
    backbuffer_rtv: *mut ID3D11RenderTargetView,
    backbuffer_texture: *mut ID3D11Texture2D,

    vertex_shader: *mut ID3D11VertexShader,
    pixel_shader: *mut ID3D11PixelShader,
    command_context: *mut ID3D11DeviceContext1,
}

fn create_device_graphics_layer(hwnd: HWND) -> Result<GraphicsDeviceLayer, ()> {
    unsafe {
        // use default adapter
        let adapter: *mut IDXGIAdapter = std::ptr::null_mut();
        let flags: UINT = 0;

        let feature_levels: D3D_FEATURE_LEVEL = D3D_FEATURE_LEVEL_11_0;
        let num_feature_levels: UINT = 1;

        let mut d3d11_device: *mut ID3D11Device = std::ptr::null_mut();
        let mut d3d11_immediate_context: *mut ID3D11DeviceContext = std::ptr::null_mut();

        let result: HRESULT = D3D11CreateDevice(
            adapter,
            D3D_DRIVER_TYPE_HARDWARE,
            std::ptr::null_mut(),
            flags,
            &feature_levels,
            num_feature_levels,
            D3D11_SDK_VERSION,
            &mut d3d11_device,
            std::ptr::null_mut(),
            &mut d3d11_immediate_context,
        );

        assert!(
            result == winapi::shared::winerror::S_OK,
            "d3d11 device creation failed"
        );

        let mut dxgi_device: *mut IDXGIDevice = std::ptr::null_mut();

        // get dxgi device
        let result = d3d11_device.as_ref().unwrap().QueryInterface(
            &IDXGIDevice::uuidof(),
            &mut dxgi_device as *mut *mut IDXGIDevice as *mut *mut winapi::ctypes::c_void,
        );

        assert!(
            result == winapi::shared::winerror::S_OK,
            "QueryInterface failed"
        );

        let mut dxgi_adapter: *mut IDXGIAdapter = std::ptr::null_mut();
        let result = dxgi_device.as_ref().unwrap().GetAdapter(&mut dxgi_adapter);

        assert!(
            result == winapi::shared::winerror::S_OK,
            "GetAdapter failed"
        );

        let mut dxgi_factory: *mut IDXGIFactory1 = std::ptr::null_mut();

        let result = dxgi_adapter.as_ref().unwrap().GetParent(
            &IDXGIFactory1::uuidof(),
            &mut dxgi_factory as *mut *mut IDXGIFactory1 as *mut *mut winapi::ctypes::c_void,
        );

        assert!(result == winapi::shared::winerror::S_OK, "GetParent failed");

        let mut dxgi_factory_2: *mut IDXGIFactory2 = std::ptr::null_mut();

        let result = dxgi_factory.as_ref().unwrap().QueryInterface(
            &IDXGIFactory2::uuidof(),
            &mut dxgi_factory_2 as *mut *mut IDXGIFactory2 as *mut *mut winapi::ctypes::c_void,
        );

        assert!(
            result == winapi::shared::winerror::S_OK,
            "dxgi_factory QueryInterface failed"
        );

        let sd = DXGI_SWAP_CHAIN_DESC1 {
            Width: 0,
            Height: 0,
            Format: DXGI_FORMAT_R8G8B8A8_UNORM,
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
            BufferCount: 2,
            AlphaMode: DXGI_ALPHA_MODE_UNSPECIFIED,
            Flags: 0,
            Scaling: DXGI_SCALING_STRETCH,
            SwapEffect: DXGI_SWAP_EFFECT_DISCARD,
            Stereo: 0,
        };

        let mut swapchain: *mut IDXGISwapChain1 = std::ptr::null_mut();

        let result = dxgi_factory_2.as_ref().unwrap().CreateSwapChainForHwnd(
            d3d11_device as *mut winapi::um::unknwnbase::IUnknown,
            hwnd,
            &sd,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut swapchain,
        );

        assert!(
            result == winapi::shared::winerror::S_OK,
            "CreateSwapChainForHwnd failed"
        );

        let mut backbuffer_texture: *mut ID3D11Texture2D = std::ptr::null_mut();
        swapchain.as_ref().unwrap().GetBuffer(
            0,
            &ID3D11Texture2D::uuidof(),
            &mut backbuffer_texture as *mut *mut ID3D11Texture2D
                as *mut *mut winapi::ctypes::c_void,
        );

        let mut backbuffer_rtv: *mut ID3D11RenderTargetView = std::ptr::null_mut();

        // now create a render target view onto the texture
        d3d11_device.as_ref().unwrap().CreateRenderTargetView(
            backbuffer_texture as *mut winapi::um::d3d11::ID3D11Resource,
            std::ptr::null_mut(),
            &mut backbuffer_rtv,
        );

        let mut command_context: *mut ID3D11DeviceContext = std::ptr::null_mut();
        let mut command_context1: *mut ID3D11DeviceContext1 = std::ptr::null_mut();

        let error = d3d11_device
            .as_ref()
            .unwrap()
            .CreateDeferredContext(0, &mut command_context);

        assert!(error == winapi::shared::winerror::S_OK);

        command_context.as_ref().unwrap().QueryInterface(
            &ID3D11DeviceContext1::uuidof(),
            &mut command_context1 as *mut *mut ID3D11DeviceContext1
                as *mut *mut winapi::ctypes::c_void,
        );

        assert!(error == winapi::shared::winerror::S_OK);

        // release the old interface, we don't need it anymore.
        // all further access will be done via the ID3D11DeviceContext1 interface
        command_context.as_ref().unwrap().Release();

        let mut vertex_shader: *mut ID3D11VertexShader = std::ptr::null_mut();
        let mut pixel_shader: *mut ID3D11PixelShader = std::ptr::null_mut();

        // load a shader
        let vertex_shader_memory =
            std::fs::read("target_data/shaders/screen_space_quad.vsb").unwrap();
        let pixel_shader_memory =
            std::fs::read("target_data/shaders/screen_space_quad.psb").unwrap();

        let error: HRESULT = d3d11_device.as_ref().unwrap().CreateVertexShader(
            vertex_shader_memory.as_ptr() as *const winapi::ctypes::c_void,
            vertex_shader_memory.len(),
            std::ptr::null_mut(),
            &mut vertex_shader as *mut *mut ID3D11VertexShader,
        );

        assert!(error == winapi::shared::winerror::S_OK);

        let error: HRESULT = d3d11_device.as_ref().unwrap().CreatePixelShader(
            pixel_shader_memory.as_ptr() as *const winapi::ctypes::c_void,
            pixel_shader_memory.len(),
            std::ptr::null_mut(),
            &mut pixel_shader as *mut *mut ID3D11PixelShader,
        );

        assert!(error == winapi::shared::winerror::S_OK);

        Ok(GraphicsDeviceLayer {
            device: d3d11_device,
            immediate_context: d3d11_immediate_context,
            swapchain,
            backbuffer_texture,
            backbuffer_rtv,
            vertex_shader,
            pixel_shader,
            command_context: command_context1,
        })
    }
}

fn main() {
    let mut should_game_close = false;

    // afterwards open a window we can render into
    let main_window: Window = create_window().unwrap();

    let graphics_layer: GraphicsDeviceLayer =
        create_device_graphics_layer(main_window.hwnd).unwrap();

    // create data required for each frame
    let cpu_render_frame_data: [CpuRenderFrameData; 2] = [
        CpuRenderFrameData {
            frame_constant_buffer: create_constant_buffer(
                unsafe { graphics_layer.device.as_ref().unwrap() },
                1024 * 8,
            ),
        },
        CpuRenderFrameData {
            frame_constant_buffer: create_constant_buffer(
                unsafe { graphics_layer.device.as_ref().unwrap() },
                1024 * 8,
            ),
        },
    ];

    let dt: f32 = 1.0 / 60.0;
    let mut accumulator: f32 = dt;

    let mut current_time = std::time::Instant::now();
    let mut draw_frame_number: u64 = 0;
    let mut update_frame_number: u64 = 0;

    let mut timer_update = 0.0;

    while !should_game_close {
        let new_time = std::time::Instant::now();

        while let Some(x) = process_window_messages(&main_window) {
            match x {
                WindowMessages::WindowClosed => {
                    should_game_close = true;
                }
                WindowMessages::WindowCreated(_x) => {
                    panic!();
                } // this should never happen
            }
        }

        // calculate how much time has passed
        let frame_time = f32::min(
            as_fractional_secs(&new_time.duration_since(current_time)),
            0.25,
        );

        accumulator += frame_time;

        println!("frame time {}", frame_time);

        current_time = new_time;

        while accumulator >= dt {
            println!(
                "update {} accumulator {} dt {} ",
                update_frame_number, accumulator, dt
            );
            timer_update += dt;

            // update the game for a fixed number of steps
            accumulator -= dt;
            update_frame_number += 1;
        }

        // draw the game
        let subframe_blend = accumulator / dt;

        let timer_draw = timer_update + accumulator;

        // draw
        println!(
            "draw {} subframe_blend {}",
            draw_frame_number, subframe_blend
        );

        let color: [f32; 4] = [0.0, 0.2, 0.4, 1.0];
        unsafe {
            let frame_data: &CpuRenderFrameData =
                &cpu_render_frame_data[draw_frame_number as usize % cpu_render_frame_data.len()];

            let constant_buffer = frame_data
                .frame_constant_buffer
                .native_buffer
                .as_mut()
                .unwrap();

            let mut gpu_heap = LinearAllocator {
                gpu_data: map_gpu_buffer(
                    constant_buffer,
                    graphics_layer.immediate_context.as_ref().unwrap(),
                ),
                state: LinearAllocatorState { used_bytes: 0 },
            };

            let command_context = graphics_layer.command_context.as_ref().unwrap();

            command_context.ClearRenderTargetView(graphics_layer.backbuffer_rtv, &color);

            let viewport: D3D11_VIEWPORT = D3D11_VIEWPORT {
                Height: 400.0,
                Width: 400.0,
                MinDepth: 0.0,
                MaxDepth: 1.0,
                TopLeftX: 0.0,
                TopLeftY: 0.0,
            };

            // set viewport for the output window
            command_context.RSSetViewports(1, &viewport);

            // bind backbuffer as render target
            let rtvs: [*mut winapi::um::d3d11::ID3D11RenderTargetView; 1] =
                [graphics_layer.backbuffer_rtv];
            command_context.OMSetRenderTargets(1, rtvs.as_ptr(), std::ptr::null_mut());

            let cycle_length_seconds = 2.0;

            let color = Float3 {
                x: f32::sin(2.0 * std::f32::consts::PI * (timer_draw / cycle_length_seconds)) * 0.5
                    + 0.5,
                y: 0.0,
                z: 0.0,
            };

            // allocate the constants for this draw call
            let obj1_alloc: HeapAlloc<ScreenSpaceQuadData> = HeapAlloc::new(
                ScreenSpaceQuadData {
                    color,
                    padding: 0.0,
                    scale: Float2 { x: 0.5, y: 0.5 },
                    position: Float2 { x: 0.0, y: 0.0 },
                },
                &gpu_heap.gpu_data,
                &mut gpu_heap.state,
            );

            // bind the shaders
            command_context.VSSetShader(graphics_layer.vertex_shader, std::ptr::null_mut(), 0);
            command_context.PSSetShader(graphics_layer.pixel_shader, std::ptr::null_mut(), 0);

            let first_constant: u32 = obj1_alloc.first_constant_offset;
            let num_constants: u32 = obj1_alloc.num_constants;

            let buffers: [*mut ID3D11Buffer; 1] = [std::ptr::null_mut()];

            command_context.VSSetConstantBuffers(
                0, // which slot to bind to
                1, // the number of buffers to bind
                buffers.as_ptr(),
            );

            command_context.PSSetConstantBuffers(
                0, // which slot to bind to
                1, // the number of buffers to bind
                buffers.as_ptr(),
            );

            command_context.PSSetConstantBuffers1(
                0,                                               // which slot to bind to
                1,                                               // the number of buffers to bind
                &frame_data.frame_constant_buffer.native_buffer, // the buffer to bind
                &first_constant,
                &num_constants,
            );

            command_context.VSSetConstantBuffers1(
                0,                                               // which slot to bind to
                1,                                               // the number of buffers to bind
                &frame_data.frame_constant_buffer.native_buffer, // the buffer to bind
                &first_constant,
                &num_constants,
            );

            // we are drawing 4 vertices using a triangle strip topology
            command_context.IASetPrimitiveTopology(D3D11_PRIMITIVE_TOPOLOGY_TRIANGLESTRIP);
            command_context.Draw(4, 0);

            // unmap the gpu buffer
            // from this point onwards we are unable to allocate further memory
            unmap_gpu_buffer(
                gpu_heap.gpu_data,
                graphics_layer.immediate_context.as_ref().unwrap(),
            );

            let mut command_list: *mut ID3D11CommandList = std::ptr::null_mut();

            let result = command_context.FinishCommandList(0, &mut command_list);

            assert!(result == winapi::shared::winerror::S_OK);

            graphics_layer
                .immediate_context
                .as_ref()
                .unwrap()
                .ExecuteCommandList(command_list, 1);

            // once the command list is executed, we can release it
            command_list.as_ref().unwrap().Release();
        }

        unsafe {
            graphics_layer.swapchain.as_ref().unwrap().Present(1, 0);
        }

        draw_frame_number += 1;
    }

    unsafe {
        for frame_data in &cpu_render_frame_data {
            frame_data
                .frame_constant_buffer
                .native_buffer
                .as_ref()
                .unwrap()
                .Release();
        }

        graphics_layer.backbuffer_rtv.as_ref().unwrap().Release();
        graphics_layer
            .backbuffer_texture
            .as_ref()
            .unwrap()
            .Release();

        graphics_layer.immediate_context.as_ref().unwrap().Release();
        graphics_layer.swapchain.as_ref().unwrap().Release();
        graphics_layer.device.as_ref().unwrap().Release();
    }
}
