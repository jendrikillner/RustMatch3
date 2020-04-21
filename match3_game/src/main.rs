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
#[derive(Clone)]
struct FrameParams {
    game_state_stack: std::vec::Vec<GameStates>,

    // this value will be set with the information for the next game state we are going to transition to
    m_next_game_state: Option<GameStates>,
}

#[derive(Copy, Clone)]
struct GameplayFrameParams {
    // the state of the grid
    grid: [[bool; 5]; 6],
}

#[derive(Copy, Clone)]
enum GameStates {
    Gameplay(GameplayFrameParams),
}

fn update_gameplay_state(frame_params: &mut GameplayFrameParams, messages: &Vec<WindowMessages>) {
    let rnd_row = 5;
    let rnd_col = 4;

    for x in messages {
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
                panic!();
            } // this should never happen, handled by higher level code
            WindowMessages::WindowCreated(_x) => {
                panic!();
            } // this should never happen
        }
    }
}

fn draw_gameplay_state(
    frame_params: &GameplayFrameParams,
    command_list: &mut GraphicsCommandList,
    backbuffer_rtv: &RenderTargetView,
    screenspace_quad_pso: &PipelineStateObject,
    gpu_heap_data: &MappedGpuData,
    gpu_heap_state: &mut LinearAllocatorState,
) {
    // draw

    let color: [f32; 4] = [0.0, 0.2, 0.4, 1.0];

    begin_render_pass(command_list, color, backbuffer_rtv);

    bind_pso(command_list, &screenspace_quad_pso);

    for (y, row) in frame_params.grid.iter().enumerate() {
        for (x, column) in row.iter().enumerate() {
            let x_offset_in_pixels = (x as f32) * 180.0;
            let y_offset_in_pixels = (y as f32) * 180.0;

            // allocate the constants for this draw call
            let obj_alloc = HeapAlloc::new(
                ScreenSpaceQuadData {
                    color: if !column {
                        Float3 {
                            x: 1.0,
                            y: 0.0,
                            z: 0.0,
                        }
                    } else {
                        Float3 {
                            x: 0.0,
                            y: 1.0,
                            z: 0.0,
                        }
                    },
                    padding: 0.0,
                    scale: Float2 {
                        x: (90.0 / 540.0),
                        y: (90.0 / 960.0),
                    },
                    position: Float2 {
                        x: (90.0 / 540.0) * -4.0 + x_offset_in_pixels / 540.0,
                        y: (90.0 / 960.0) * 6.0 - y_offset_in_pixels / 960.0,
                    },
                },
                gpu_heap_data,
                gpu_heap_state,
            );

            bind_constant(command_list, 0, &obj_alloc);

            draw_vertices(command_list, 4);
        }
    }
}

fn main() {
    let args: CommandLineArgs = parse_cmdline();

    let mut should_game_close = false;

    // afterwards open a window we can render into
    let main_window: Window = create_window(540, 960).unwrap();

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
        m_next_game_state: Some(GameStates::Gameplay(GameplayFrameParams {
            grid: { [[false; 5]; 6] },
        })),
        game_state_stack: Vec::new(),
    };

    let mut frame_params1 = frame_params0.clone();

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

        // clone the previous frame state as starting point for the next frame
        *frame_params = prev_frame_params.clone();

        while accumulator >= dt {
            // update the game for a fixed number of steps
            accumulator -= dt;

            // before we start to update the game check if a new gameplay state needs to be created
            if frame_params.m_next_game_state.is_some() {
                // waiting game state
                // copy the state from the waiting into the active list
                frame_params
                    .game_state_stack
                    .push(frame_params.m_next_game_state.unwrap());

                frame_params.m_next_game_state = None;
            }

            let mut messages: Vec<WindowMessages> = Vec::new();

            while let Some(x) = process_window_messages(&main_window) {
                match x {
                    WindowMessages::WindowClosed => {
                        should_game_close = true;
                    }
                    WindowMessages::WindowCreated(_x) => {
                        panic!();
                    } // this should never happen
                    _ => messages.push(x),
                }
            }

            // iterative over all game states
            for game_state in frame_params.game_state_stack.iter_mut().rev() {
                match game_state {
                    GameStates::Gameplay(state) => update_gameplay_state(state, &messages),
                }
            }

            update_frame_number += 1;
        }

        // draw the game
        let frame_data: &CpuRenderFrameData =
            &cpu_render_frame_data[draw_frame_number as usize % cpu_render_frame_data.len()];

        let mut gpu_heap = LinearAllocator {
            gpu_data: map_gpu_buffer(&frame_data.frame_constant_buffer, &graphics_layer),
            state: LinearAllocatorState { used_bytes: 0 },
        };

        for game_state in frame_params.game_state_stack.iter_mut() {
            match game_state {
                GameStates::Gameplay(state) => draw_gameplay_state(&state, & mut graphics_layer.graphics_command_list, & graphics_layer.backbuffer_rtv, &screenspace_quad_pso, & gpu_heap.gpu_data, &mut gpu_heap.state ),
            }
        }

        // unmap the gpu buffer
        // from this point onwards we are unable to allocate further memory
        unmap_gpu_buffer(gpu_heap.gpu_data, &graphics_layer);

        execute_command_list(&graphics_layer, &graphics_layer.graphics_command_list);

        present_swapchain(&graphics_layer);

        draw_frame_number += 1;
    }
}
