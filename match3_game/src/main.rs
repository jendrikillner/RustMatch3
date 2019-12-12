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
use winapi::um::d3dcommon::*;
use winapi::um::winuser::*;
use winapi::Interface;

fn as_fractional_secs(dur: &std::time::Duration) -> f32 {
    (dur.as_secs() as f64 + f64::from(dur.subsec_nanos()) / 1_000_000_000.0) as f32
}

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

struct GraphicsDeviceLayer {
    device: *mut ID3D11Device,
    immediate_context: *mut ID3D11DeviceContext,
    swapchain: *mut IDXGISwapChain1,
    backbuffer_rtv: *mut ID3D11RenderTargetView,
    backbuffer_texture: *mut ID3D11Texture2D,
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

        Ok(GraphicsDeviceLayer {
            device: d3d11_device,
            immediate_context: d3d11_immediate_context,
            swapchain,
            backbuffer_texture,
            backbuffer_rtv,
        })
    }
}

fn main() {
    let mut should_game_close = false;

    // afterwards open a window we can render into
    let main_window: Window = create_window().unwrap();

    let graphics_layer: GraphicsDeviceLayer =
        create_device_graphics_layer(main_window.hwnd).unwrap();

    let dt: f32 = 1.0 / 60.0;
    let mut accumulator: f32 = dt;

    let mut current_time = std::time::Instant::now();
    let mut draw_frame_number: u64 = 0;
    let mut update_frame_number: u64 = 0;

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

            // update the game for a fixed number of steps
            accumulator -= dt;
            update_frame_number += 1;
        }

        // draw the game
        let subframe_blend = accumulator / dt;

        // draw
        println!(
            "draw {} subframe_blend {}",
            draw_frame_number, subframe_blend
        );

        let color: [f32; 4] = [0.0, 0.2, 0.4, 1.0];
        unsafe {
            graphics_layer.immediate_context
                .as_ref()
                .unwrap()
                .ClearRenderTargetView(graphics_layer.backbuffer_rtv, &color);
        }

        unsafe {
            graphics_layer.swapchain.as_ref().unwrap().Present(1, 0);
        }

        draw_frame_number += 1;
    }

    unsafe {
        graphics_layer.backbuffer_rtv.as_ref().unwrap().Release();
        graphics_layer.backbuffer_texture.as_ref().unwrap().Release();

        graphics_layer.immediate_context.as_ref().unwrap().Release();
        graphics_layer.swapchain.as_ref().unwrap().Release();
        graphics_layer.device.as_ref().unwrap().Release();
    }
}
