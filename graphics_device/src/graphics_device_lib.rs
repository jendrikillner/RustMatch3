use winapi::shared::dxgi::*;
use winapi::shared::dxgi1_2::*;
use winapi::shared::dxgiformat::*;
use winapi::shared::dxgitype::*;
use winapi::shared::minwindef::{UINT, ULONG};
use winapi::shared::ntdef::HRESULT;
use winapi::shared::windef::HWND;
use winapi::um::d3d11::*;
use winapi::um::d3d11_1::*;
use winapi::um::d3d11sdklayers::*;
use winapi::um::d3dcommon::*;
use winapi::Interface;

pub fn leak_check_release(
    object_to_release: &winapi::um::unknwnbase::IUnknown,
    expected_ref_count: ULONG,
    debug_device: Option<&ID3D11Debug>,
) {
    let prev_refcount: ULONG = unsafe { object_to_release.Release() };

    if prev_refcount == expected_ref_count {
        return;
    }

    // if we are runnign with the debug device, log the outstanding references
    if let Some(x) = debug_device {
        unsafe {
            x.ReportLiveDeviceObjects(D3D11_RLDO_DETAIL);
        }
    }

    assert!(
        prev_refcount == expected_ref_count,
        "object was not released, still has {} outstanding references, expected {} ",
        prev_refcount,
        expected_ref_count
    );
}

fn set_debug_name(device_child: &ID3D11DeviceChild, name: &str) {
    unsafe {
        device_child.SetPrivateData(
            &WKPDID_D3DDebugObjectName,
            name.len() as u32,
            name.as_ptr() as *const winapi::ctypes::c_void,
        );
    }
}

pub struct MappedGpuData<'a> {
    data: &'a [u8],        // reference to slice of cpu accessible gpu memory
    buffer: &'a GpuBuffer, // reference to the d3d11 buffer the data comes from
}

pub fn map_gpu_buffer<'a>(
    buffer: &'a GpuBuffer,
    device_layer: &GraphicsDeviceLayer,
) -> MappedGpuData<'a> {
    let mut mapped_resource = D3D11_MAPPED_SUBRESOURCE {
        pData: std::ptr::null_mut(),
        RowPitch: 0,
        DepthPitch: 0,
    };

    let native_buffer: &mut ID3D11Buffer = unsafe { buffer.native_buffer.as_mut().unwrap() };

    // map the buffer
    let result: HRESULT = unsafe {
        device_layer.immediate_context.as_ref().unwrap().Map(
            native_buffer as *mut ID3D11Buffer as *mut winapi::um::d3d11::ID3D11Resource,
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

pub fn unmap_gpu_buffer(mapped_data: MappedGpuData, device_layer: &GraphicsDeviceLayer) {
    unsafe {
        device_layer.immediate_context.as_ref().unwrap().Unmap(
            mapped_data.buffer.native_buffer as *mut ID3D11Buffer
                as *mut winapi::um::d3d11::ID3D11Resource,
            0,
        );
    }
}

pub struct LinearAllocatorState {
    pub used_bytes: usize,
}

pub struct LinearAllocator<'a> {
    pub gpu_data: MappedGpuData<'a>,

    pub state: LinearAllocatorState,
}

pub struct HeapAlloc<'a> {
    gpu_buffer_src: &'a GpuBuffer,
    pub first_constant_offset: u32,
    pub num_constants: u32,
}

pub fn round_up_to_multiple(number: usize, multiple: usize) -> usize {
    ((number + multiple - 1) / multiple) * multiple
}

impl<'a> HeapAlloc<'a> {
    pub fn new<T>(
        x: T,
        gpu_data: &'a MappedGpuData,
        state: &mut LinearAllocatorState,
    ) -> HeapAlloc<'a> {
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
                gpu_buffer_src: gpu_data.buffer,
                first_constant_offset: (start_offset_in_bytes / 16) as u32,
                num_constants: (allocation_size / 16) as u32,
            }
        }
    }
}

pub struct GpuBuffer {
    pub native_buffer: *mut ID3D11Buffer,
}

impl Drop for GpuBuffer {
    fn drop(&mut self) {
        leak_check_release(unsafe { self.native_buffer.as_ref().unwrap() }, 0, None);
    }
}

pub fn create_constant_buffer(
    device_layer: &GraphicsDeviceLayer,
    size_in_bytes: u32,
    debug_name: &str,
) -> GpuBuffer {
    let mut constant_buffer: *mut ID3D11Buffer = std::ptr::null_mut();

    let buffer_desc = D3D11_BUFFER_DESC {
        ByteWidth: size_in_bytes,
        Usage: D3D11_USAGE_DYNAMIC,
        BindFlags: D3D11_BIND_CONSTANT_BUFFER,
        CPUAccessFlags: D3D11_CPU_ACCESS_WRITE,
        MiscFlags: 0,
        StructureByteStride: 0,
    };

    let error = unsafe {
        device_layer.device.native.CreateBuffer(
            &buffer_desc,
            std::ptr::null(),
            &mut constant_buffer,
        )
    };

    assert!(error == winapi::shared::winerror::S_OK);

    unsafe {
        set_debug_name(
            constant_buffer.as_ref().unwrap(),
            format!("Constant Buffer - {}", debug_name).as_str(),
        );
    }

    GpuBuffer {
        native_buffer: constant_buffer,
    }
}

pub struct GraphicsCommandList<'a> {
    pub command_context: *mut ID3D11DeviceContext1,
    phantom: std::marker::PhantomData<&'a mut ID3D11DeviceContext1>, // a marker to indicate that we are holding a reference to ID3D11DeviceContext1 evenso we store a pointer. This is required for lifetime tracking
}

impl Drop for GraphicsCommandList<'_> {
    fn drop(&mut self) {
        unsafe {
            leak_check_release(self.command_context.as_ref().unwrap(), 0, None);
        }
    }
}

pub struct RenderTargetView<'a> {
    pub native_view: &'a mut winapi::um::d3d11::ID3D11RenderTargetView,
	width : i32,
	height : i32
}

impl Drop for RenderTargetView<'_> {
    fn drop(&mut self) {
        leak_check_release(self.native_view, 0, None);
    }
}

pub struct GraphicsDevice<'a> {
    pub native: &'a mut ID3D11Device,
    pub debug_device: Option<&'a ID3D11Debug>,
}

impl Drop for GraphicsDevice<'_> {
    fn drop(&mut self) {
        let expected_device_ref_count = if self.debug_device.is_some() { 1 } else { 0 };

        leak_check_release(self.native, expected_device_ref_count, self.debug_device);

        if let Some(x) = self.debug_device {
            leak_check_release(&x, 0, None);
        }
    }
}

pub struct GraphicsDeviceLayer<'a> {
    pub immediate_context: *mut ID3D11DeviceContext,
    pub swapchain: *mut IDXGISwapChain1,
    pub backbuffer_texture: *mut ID3D11Texture2D,

    pub backbuffer_rtv: RenderTargetView<'a>,
    pub graphics_command_list: GraphicsCommandList<'a>,

    // this needs to be the last parameter to make sure that all items that depend on ID3D11Device have been dropped before the device is dropped
    pub device: GraphicsDevice<'a>,
}

impl Drop for GraphicsDeviceLayer<'_> {
    fn drop(&mut self) {
        unsafe {
            leak_check_release(
                self.backbuffer_texture.as_ref().unwrap(),
                0,
                self.device.debug_device,
            );
            leak_check_release(
                self.immediate_context.as_ref().unwrap(),
                0,
                self.device.debug_device,
            );
            leak_check_release(
                self.swapchain.as_ref().unwrap(),
                0,
                self.device.debug_device,
            );
        }
    }
}

pub fn create_device_graphics_layer<'a>(
    hwnd: HWND,
    enable_debug_device: bool,
) -> Result<GraphicsDeviceLayer<'a>, ()> {
    unsafe {
        // use default adapter
        let adapter: *mut IDXGIAdapter = std::ptr::null_mut();

        let flags: UINT = if enable_debug_device {
            D3D11_CREATE_DEVICE_DEBUG
        } else {
            0
        };

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

        set_debug_name(
            d3d11_immediate_context.as_ref().unwrap(),
            "Immediate Context",
        );

        let mut debug_device: *mut ID3D11Debug = std::ptr::null_mut();

        if enable_debug_device {
            // get d3d11 debug devuce
            d3d11_device.as_ref().unwrap().QueryInterface(
                &ID3D11Debug::uuidof(),
                &mut debug_device as *mut *mut ID3D11Debug as *mut *mut winapi::ctypes::c_void,
            );
        }

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
            SwapEffect: DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL,
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

        set_debug_name(backbuffer_texture.as_ref().unwrap(), "Backbuffer Texture");

        let mut backbuffer_rtv: *mut ID3D11RenderTargetView = std::ptr::null_mut();

        // now create a render target view onto the texture
        d3d11_device.as_ref().unwrap().CreateRenderTargetView(
            backbuffer_texture as *mut winapi::um::d3d11::ID3D11Resource,
            std::ptr::null_mut(),
            &mut backbuffer_rtv,
        );

        set_debug_name(backbuffer_rtv.as_ref().unwrap(), "Backbuffer RTV");

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

        // should keep a ref-count of 1 because they are alternative views onto objects that have another view that is still active
        leak_check_release(command_context.as_ref().unwrap(), 1, debug_device.as_ref());
        dxgi_device.as_ref().unwrap().Release();

        set_debug_name(command_context.as_ref().unwrap(), "Deferred Context");

		let mut rect = winapi::shared::windef::RECT { bottom: 0, left: 0, right: 0, top: 0 };

		winapi::um::winuser::GetClientRect(
		  hwnd,
		  &mut rect
		);

        Ok(GraphicsDeviceLayer {
            device: GraphicsDevice {
                native: d3d11_device.as_mut().unwrap(),
                debug_device: debug_device.as_ref(),
            },
            immediate_context: d3d11_immediate_context,
            swapchain,
            backbuffer_texture,
            backbuffer_rtv: RenderTargetView {
                native_view: backbuffer_rtv.as_mut().unwrap(),
				width : rect.right,
				height: rect.bottom,
            },
            graphics_command_list: GraphicsCommandList {
                command_context: command_context1,
                phantom: std::marker::PhantomData,
            },
        })
    }
}

#[derive(Debug)]
pub struct PipelineStateObjectDesc<'a> {
    pub shader_name: &'a str,
}

pub struct PipelineStateObject<'a> {
    pub vertex_shader: &'a ID3D11VertexShader,
    pub pixel_shader: &'a ID3D11PixelShader,
}

impl Drop for PipelineStateObject<'_> {
    fn drop(&mut self) {
        leak_check_release(self.vertex_shader, 0, None);
        leak_check_release(self.pixel_shader, 0, None);
    }
}

pub fn create_pso<'a>(
    device: &GraphicsDevice,
    desc: PipelineStateObjectDesc,
) -> PipelineStateObject<'a> {
    // build the name of the vertex and pixel shader to load
    let vertex_shader_name = format!("{0}.vsb", desc.shader_name);
    let pixel_shader_name = format!("{0}.psb", desc.shader_name);

    let mut vertex_shader: *mut ID3D11VertexShader = std::ptr::null_mut();
    let mut pixel_shader: *mut ID3D11PixelShader = std::ptr::null_mut();

    // load a shader
    let vertex_shader_memory = std::fs::read(&vertex_shader_name).unwrap();
    let pixel_shader_memory = std::fs::read(&pixel_shader_name).unwrap();

    let error: HRESULT = unsafe {
        device.native.CreateVertexShader(
            vertex_shader_memory.as_ptr() as *const winapi::ctypes::c_void,
            vertex_shader_memory.len(),
            std::ptr::null_mut(),
            &mut vertex_shader as *mut *mut ID3D11VertexShader,
        )
    };

    assert!(error == winapi::shared::winerror::S_OK);

    unsafe {
        set_debug_name(
            vertex_shader.as_ref().unwrap(),
            format!("PSO [{:?}] src-file: {1}", &desc, &vertex_shader_name).as_str(),
        );
    }

    let error: HRESULT = unsafe {
        device.native.CreatePixelShader(
            pixel_shader_memory.as_ptr() as *const winapi::ctypes::c_void,
            pixel_shader_memory.len(),
            std::ptr::null_mut(),
            &mut pixel_shader as *mut *mut ID3D11PixelShader,
        )
    };

    assert!(error == winapi::shared::winerror::S_OK);

    unsafe {
        set_debug_name(
            pixel_shader.as_ref().unwrap(),
            format!("PSO [{:?}] src-file: {1}", &desc, &pixel_shader_name).as_str(),
        );
    }

    PipelineStateObject {
        vertex_shader: unsafe { vertex_shader.as_mut().unwrap() },
        pixel_shader: unsafe { pixel_shader.as_mut().unwrap() },
    }
}

pub fn begin_render_pass(
    command_list: &mut GraphicsCommandList,
    clear_color: [f32; 4],
    rtv: &RenderTargetView,
) {
    unsafe {
        let command_context = command_list.command_context.as_ref().unwrap();

        let rtv_mut: *mut ID3D11RenderTargetView =
            rtv.native_view as *const ID3D11RenderTargetView as u64 as *mut ID3D11RenderTargetView;

        command_context.ClearRenderTargetView(rtv_mut, &clear_color);

        let viewport: D3D11_VIEWPORT = D3D11_VIEWPORT {
            Height: rtv.height as f32,
            Width:  rtv.width as f32,
            MinDepth: 0.0,
            MaxDepth: 1.0,
            TopLeftX: 0.0,
            TopLeftY: 0.0,
        };

        // set viewport for the output window
        command_context.RSSetViewports(1, &viewport);

        // bind backbuffer as render target
        let rtvs: [*mut winapi::um::d3d11::ID3D11RenderTargetView; 1] = [rtv_mut];
        command_context.OMSetRenderTargets(1, rtvs.as_ptr(), std::ptr::null_mut());
    }
}

pub fn bind_pso(command_list: &mut GraphicsCommandList, pso: &PipelineStateObject) {
    unsafe {
        let command_context = command_list.command_context.as_ref().unwrap();

        // hack around the fact that VSSetShader takes a mutable pointer
        // the function never modifies the vertex or pixel shader
        // don't want the interface to have to expose mutable PipelineStateObject references because of it
        // instead take the poiner value, read the absolute u64 value of the adress and cast that to a mutable pointer
        // sorry borrow checker :)
        let vertex_shader_mut: *mut ID3D11VertexShader =
            (pso.vertex_shader as *const ID3D11VertexShader as u64) as *mut ID3D11VertexShader;
        let pixel_shader_mut: *mut ID3D11PixelShader =
            (pso.pixel_shader as *const ID3D11PixelShader as u64) as *mut ID3D11PixelShader;

        // bind the shaders
        command_context.VSSetShader(vertex_shader_mut, std::ptr::null_mut(), 0);
        command_context.PSSetShader(pixel_shader_mut, std::ptr::null_mut(), 0);

        // fow now assume all PSO will be using this state
        command_context.IASetPrimitiveTopology(D3D11_PRIMITIVE_TOPOLOGY_TRIANGLESTRIP);
    }
}

pub fn bind_constant(
    command_list: &mut GraphicsCommandList,
    bind_slot: u32,
    constant_alloc: &HeapAlloc,
) {
    let command_context = unsafe { command_list.command_context.as_ref().unwrap() };

    let first_constant: u32 = constant_alloc.first_constant_offset;
    let num_constants: u32 = constant_alloc.num_constants;

    let null_buffers: [*mut ID3D11Buffer; 1] = [std::ptr::null_mut()];
    let buffers: [*mut ID3D11Buffer; 1] = [constant_alloc.gpu_buffer_src.native_buffer];

    unsafe {
        command_context.VSSetConstantBuffers(
            0, // which slot to bind to
            1, // the number of buffers to bind
            null_buffers.as_ptr(),
        );

        command_context.PSSetConstantBuffers(
            0, // which slot to bind to
            1, // the number of buffers to bind
            null_buffers.as_ptr(),
        );

        command_context.PSSetConstantBuffers1(
            bind_slot,        // which slot to bind to
            1,                // the number of buffers to bind
            buffers.as_ptr(), // the buffer to bind
            &first_constant,
            &num_constants,
        );

        command_context.VSSetConstantBuffers1(
            bind_slot,        // which slot to bind to
            1,                // the number of buffers to bind
            buffers.as_ptr(), // the buffer to bind
            &first_constant,
            &num_constants,
        );
    }
}

pub fn draw_vertices(command_list: &mut GraphicsCommandList, vertex_count: u32) {
    unsafe {
        let command_context = command_list.command_context.as_ref().unwrap();
        command_context.Draw(vertex_count, 0);
    }
}

pub fn execute_command_list(
    graphics_layer: &GraphicsDeviceLayer,
    command_list_in: &GraphicsCommandList,
) {
    unsafe {
        let command_context = command_list_in.command_context.as_ref().unwrap();

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
}

pub fn present_swapchain(graphics_layer: &GraphicsDeviceLayer) {
    unsafe {
        graphics_layer.swapchain.as_ref().unwrap().Present(1, 0);
    }
}
