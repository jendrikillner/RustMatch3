use graphics_device::*;
use os_window::*;

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

    // load the PSO required to draw the quad onto the screen

    let pso_desc = PipelineStateObjectDesc {
        shader_name: "target_data/shaders/screen_space_quad",
    };

    let screenspace_quad_pso: PipelineStateObject = create_pso(&graphics_layer.device, pso_desc);

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
            &graphics_layer.backbuffer_rtv,
        );

        let cycle_length_seconds = 2.0;

        let color = Float3 {
            x: f32::sin(2.0 * std::f32::consts::PI * (timer_draw / cycle_length_seconds)) * 0.5
                + 0.5,
            y: 0.0,
            z: 0.0,
        };

        // allocate the constants for this draw call
        let obj1_alloc = HeapAlloc::new(
            ScreenSpaceQuadData {
                color,
                padding: 0.0,
                scale: Float2 { x: 0.5, y: 0.5 },
                position: Float2 { x: 0.0, y: 0.0 },
            },
            &gpu_heap.gpu_data,
            &mut gpu_heap.state,
        );

        bind_pso(
            &mut graphics_layer.graphics_command_list,
            &screenspace_quad_pso,
        );

        bind_constant(&mut graphics_layer.graphics_command_list, 0, &obj1_alloc);

        draw_vertices(&mut graphics_layer.graphics_command_list, 4);

        // unmap the gpu buffer
        // from this point onwards we are unable to allocate further memory
        unmap_gpu_buffer(gpu_heap.gpu_data, &graphics_layer);

        execute_command_list(&graphics_layer, &graphics_layer.graphics_command_list);

        present_swapchain(&graphics_layer);

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

        graphics_layer.backbuffer_rtv.native_view.Release();
        graphics_layer
            .backbuffer_texture
            .as_ref()
            .unwrap()
            .Release();

        graphics_layer.immediate_context.as_ref().unwrap().Release();
        graphics_layer.swapchain.as_ref().unwrap().Release();
        leak_check_release(graphics_layer.device.native, 0, graphics_layer.debug_device);
    }
}
