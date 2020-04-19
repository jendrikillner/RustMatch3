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

struct CommandLineArgs {
    enable_debug_device: bool,
}

fn parse_cmdline() -> CommandLineArgs {
    let mut enable_debug_device = false;

    for argument in std::env::args() {
        // make sure we always compare agsinst the lowercase version so that casing doesn't matter
        let mut arg = argument;
        arg.make_ascii_lowercase();

        if arg == "-debugdevice" {
            enable_debug_device = true;
        }
    }

    CommandLineArgs {
        enable_debug_device,
    }
}

// data for each displayed frame
// frame = "A piece of data that is processed and ultimately displayed on screen"
struct FrameParams {
    // the state of the grid
    grid: [[bool; 5]; 6],
}

fn main() {
    let args: CommandLineArgs = parse_cmdline();

    let mut should_game_close = false;

    // afterwards open a window we can render into
    let main_window: Window = create_window().unwrap();

    let mut graphics_layer: GraphicsDeviceLayer =
        create_device_graphics_layer(main_window.hwnd, args.enable_debug_device).unwrap();

    // create data required for each frame
    let cpu_render_frame_data: [CpuRenderFrameData; 2] = [
        CpuRenderFrameData {
            frame_constant_buffer: create_constant_buffer(
                &graphics_layer,
                1024 * 8,
                "Frame 0 Constants",
            ),
        },
        CpuRenderFrameData {
            frame_constant_buffer: create_constant_buffer(
                &graphics_layer,
                1024 * 8,
                "Frame 1 Constants",
            ),
        },
    ];

    // create the initial state for the the frame params
    // frame params are passing through the following stages
    // update
    // cpu render
    // gpu render
    let mut frame_params0 = FrameParams {
        grid: { [[false; 5]; 6] },
    };

    let mut frame_params1 = FrameParams {
        grid: { [[false; 5]; 6] },
    };

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

        // at the start of the frame we allocate a new FrameParam
        // frame params are created during updated, passing through the following stages
        // update
        // cpu render
        // gpu render

        // calculate how much time has passed
        let frame_time = f32::min(
            as_fractional_secs(&new_time.duration_since(current_time)),
            0.25,
        );

        accumulator += frame_time;

        current_time = new_time;

        // for now just sleep
        // don't want to waste CPU resources rendering more frames
        if accumulator < dt {
            let sleep_duration = dt - accumulator;

            std::thread::sleep(std::time::Duration::from_secs_f32(sleep_duration));
        }

        accumulator = dt;

        let (prev_frame_params, frame_params) = if update_frame_number % 2 == 0 {
            (&frame_params1, &mut frame_params0)
        } else {
            (&frame_params0, &mut frame_params1)
        };

        let rnd_row = 3;
        let rnd_col = 4;

        frame_params.grid[rnd_row][rnd_col] = prev_frame_params.grid[rnd_row][rnd_col];

        while accumulator >= dt {
            timer_update += dt;

            // update the game for a fixed number of steps
            accumulator -= dt;

            // update the game

            while let Some(x) = process_window_messages(&main_window) {
                match x {
                    WindowMessages::MousePositionChanged(pos) => {
                        println!("cursor position changed: x {0}, y {1}", pos.x, pos.y);
                    }

                    WindowMessages::MouseLeftButtonDown => {
                        println!("mouse:left down");

                        frame_params.grid[rnd_row][rnd_col] = true;
                    }

                    WindowMessages::MouseLeftButtonUp => {
                        println!("mouse:left up");
                    }

                    WindowMessages::MouseFocusGained => {
                        println!("mouse:focus gained");
                    }

                    WindowMessages::MouseFocusLost => {
                        println!("mouse:focus lost");
                    }

                    WindowMessages::WindowClosed => {
                        should_game_close = true;
                    }
                    WindowMessages::WindowCreated(_x) => {
                        panic!();
                    } // this should never happen
                }
            }

            update_frame_number += 1;
        }

        // draw the game
        let _subframe_blend = accumulator / dt;

        let timer_draw = timer_update + accumulator;

        // draw

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
                color: if !frame_params.grid[rnd_row][rnd_col] {
                    color
                } else {
                    Float3 {
                        x: 0.0,
                        y: 1.0,
                        z: 0.0,
                    }
                },
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
}
