use winapi::um::d3d11::*;
use winapi::um::d3dcommon::*;

use os_window::*;
use graphics_device::*;

pub fn as_fractional_secs(dur: &std::time::Duration) -> f32 {
    (dur.as_secs() as f64 + f64::from(dur.subsec_nanos()) / 1_000_000_000.0) as f32
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

struct CpuRenderFrameData {
    frame_constant_buffer: GpuBuffer,
}

fn main() {
    let mut should_game_close = false;

    // afterwards open a window we can render into
    let main_window: Window = create_window().unwrap();

    let mut graphics_layer: GraphicsDeviceLayer =
        create_device_graphics_layer(main_window.hwnd).unwrap();

    // create data required for each frame
    let cpu_render_frame_data: [CpuRenderFrameData; 2] = [
        CpuRenderFrameData {
            frame_constant_buffer: create_constant_buffer(&graphics_layer, 1024 * 8),
        },
        CpuRenderFrameData {
            frame_constant_buffer: create_constant_buffer(&graphics_layer, 1024 * 8),
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
        let frame_data: &CpuRenderFrameData =
            &cpu_render_frame_data[draw_frame_number as usize % cpu_render_frame_data.len()];

        let mut gpu_heap = LinearAllocator {
            gpu_data: map_gpu_buffer(&frame_data.frame_constant_buffer, &graphics_layer),
            state: LinearAllocatorState { used_bytes: 0 },
        };

        begin_render_pass(
            &mut graphics_layer.graphics_command_list,
            color,
            graphics_layer.backbuffer_rtv,
        );

        unsafe {
            let command_context = graphics_layer.command_context.as_ref().unwrap();

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

            let null_buffers: [*mut ID3D11Buffer; 1] = [std::ptr::null_mut()];
            let buffers: [*mut ID3D11Buffer; 1] = [frame_data.frame_constant_buffer.native_buffer];

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
                0,                // which slot to bind to
                1,                // the number of buffers to bind
                buffers.as_ptr(), // the buffer to bind
                &first_constant,
                &num_constants,
            );

            command_context.VSSetConstantBuffers1(
                0,                // which slot to bind to
                1,                // the number of buffers to bind
                buffers.as_ptr(), // the buffer to bind
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
