use graphics_device::*;
use os_window::*;

pub fn as_fractional_secs(dur: &std::time::Duration) -> f32 {
    (dur.as_secs() as f64 + f64::from(dur.subsec_nanos()) / 1_000_000_000.0) as f32
}

#[repr(C)]
struct Float4 {
    x: f32,
    y: f32,
    z: f32,
	a: f32,
}

#[repr(C)]
struct Float2 {
    x: f32,
    y: f32,
}

#[repr(C)]
struct ScreenSpaceQuadData {
    color: Float4,
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

enum GameStateType {
    //MainMenu,
    Pause,
    Gameplay,
}

///  -------------------- gameplay ---------------------

struct GameplayStateStaticData {}
struct PauseStateStaticData {}

struct GameplayStateFrameData {
    // the state of the grid
    grid: [[bool; 5]; 6],
}

struct PauseStateFrameData {}

enum GameStateStaticData {
    Gameplay(GameplayStateStaticData),
    Pause(PauseStateStaticData),
}

enum GameStateFrameData {
    Gameplay(GameplayStateFrameData),
    Pause(PauseStateFrameData),
}

//struct GameState {
//	state_data : GameStateData,
//}

// data for each displayed frame
// frame = "A piece of data that is processed and ultimately displayed on screen"
struct FrameParams {
    cpu_render: CpuRenderFrameData,

    // if inside a gameplay
    gameplay_data: Vec<GameStateFrameData>,
}

fn pre_cpu_update_frame(
    frame_data: &mut GameplayStateFrameData,
    prev_frame_data: &GameplayStateFrameData,
) {
    // we could also implement the copy trait here
    // for now to keep it simple just copy the state
    // might be moved into cpu update later
    frame_data.grid = prev_frame_data.grid;
}

fn update_pause_state(
    _frame_params: &mut PauseStateFrameData,
    _messages: &Vec<WindowMessages>,
) -> Option<GameStateType> {
    None
}

fn update_gameplay_state(
    frame_params: &mut GameplayStateFrameData,
    messages: &Vec<WindowMessages>,
) -> Option<GameStateType> {
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

                return Some(GameStateType::Pause);
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

    // don't need to switch game states
    None
}

fn draw_pause_state(
    _frame_params: &PauseStateFrameData,
    command_list: &mut GraphicsCommandList,
    _backbuffer_rtv: &RenderTargetView,
    screenspace_quad_pso: &PipelineStateObject,
    gpu_heap_data: &MappedGpuData,
    gpu_heap_state: &mut LinearAllocatorState,
) {
	bind_pso(command_list, &screenspace_quad_pso);

    let obj_alloc = HeapAlloc::new(
        ScreenSpaceQuadData {
            color: Float4 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
				a: 0.8,
            },
            scale: Float2 { x: 1.0, y: 1.0 },
            position: Float2 { x: 0.0, y: 0.0 },
        },
        gpu_heap_data,
        gpu_heap_state,
    );

    bind_constant(command_list, 0, &obj_alloc);

    draw_vertices(command_list, 4);
}

fn draw_gameplay_state(
    frame_params: &GameplayStateFrameData,
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
                        Float4 {
                            x: 1.0,
                            y: 0.0,
                            z: 0.0,
							a: 1.0,
                        }
                    } else {
                        Float4 {
                            x: 0.0,
                            y: 1.0,
                            z: 0.0,
							a: 1.0,
                        }
                    },
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

// todo, update the logic todo the following
// 1. user presses the left mouse button, this will open the pause menu
// 2. the pause menu blocks input from reaching the gameplay state
// 3. enable alpha blending for the pause menu overlay
// 4. slowly blend to a dark black overlay in around 2 seconds orso
// 5. once the maximum has been reached, a left click will fade out the black screen and allow the gameplay logic to receive input again

fn main() {
    let args: CommandLineArgs = parse_cmdline();

    let mut should_game_close = false;

    // afterwards open a window we can render into
    let main_window: Window = create_window(540, 960).unwrap();

    let mut graphics_layer: GraphicsDeviceLayer =
        create_device_graphics_layer(main_window.hwnd, args.enable_debug_device).unwrap();

    let mut frame_params0 = FrameParams {
        gameplay_data: Vec::new(),
        cpu_render: CpuRenderFrameData {
            frame_constant_buffer: create_constant_buffer(
                &graphics_layer,
                1024 * 8,
                "Frame 0 Constants",
            ),
        },
    };

    let mut frame_params1 = FrameParams {
        gameplay_data: Vec::new(),
        cpu_render: CpuRenderFrameData {
            frame_constant_buffer: create_constant_buffer(
                &graphics_layer,
                1024 * 8,
                "Frame 1 Constants",
            ),
        },
    };

    // load the PSO required to draw the quad onto the screen

    let screenspace_quad_pso: PipelineStateObject = create_pso(&graphics_layer.device, PipelineStateObjectDesc {
        shader_name: "target_data/shaders/screen_space_quad",
		premultiplied_alpha: false,
    });
	let screenspace_quad_blended_pso: PipelineStateObject = create_pso(&graphics_layer.device, PipelineStateObjectDesc {
        shader_name: "target_data/shaders/screen_space_quad",
		premultiplied_alpha: true,
    });

    let dt: f32 = 1.0 / 60.0;
    let mut accumulator: f32 = dt;

    let mut current_time = std::time::Instant::now();
    let mut update_frame_number: u64 = 0;

    let mut game_state_stack: Vec<GameStateStaticData> = Vec::new();
    let mut next_game_state = Some(GameStateType::Gameplay);

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

        // we are starting a new frame, do we need to transition to a new state?
        match next_game_state {
            Some(x) => {
                match x {
                    GameStateType::Gameplay => {
                        game_state_stack
                            .push(GameStateStaticData::Gameplay(GameplayStateStaticData {}));

                        // also create the frame data in all instances of the frame data
                        frame_params0
                            .gameplay_data
                            .push(GameStateFrameData::Gameplay(GameplayStateFrameData {
                                grid: { [[false; 5]; 6] },
                            }));

                        frame_params1
                            .gameplay_data
                            .push(GameStateFrameData::Gameplay(GameplayStateFrameData {
                                grid: { [[false; 5]; 6] },
                            }));
                    }

                    GameStateType::Pause => {
                        game_state_stack.push(GameStateStaticData::Pause(PauseStateStaticData {}));

                        // also create the frame data in all instances of the frame data
                        frame_params0
                            .gameplay_data
                            .push(GameStateFrameData::Pause(PauseStateFrameData {}));

                        frame_params1
                            .gameplay_data
                            .push(GameStateFrameData::Pause(PauseStateFrameData {}));
                    }
                }

                // make sure to reset the state
                next_game_state = None;
            }

            None => {}
        }

        let (prev_frame_params, frame_params) = if update_frame_number % 2 == 0 {
            (&frame_params1, &mut frame_params0)
        } else {
            (&frame_params0, &mut frame_params1)
        };

        // iterate backwards
        // let higher level states update first
        // they will be allowed to stop input from reaching lower level states
        // and/or stop other states from updating completly
        for i in 0..frame_params.gameplay_data.len() {
            match (
                frame_params.gameplay_data.get_mut(i).unwrap(),
                prev_frame_params.gameplay_data.get(i).unwrap(),
            ) {
                (GameStateFrameData::Gameplay(x), GameStateFrameData::Gameplay(y)) => {
                    pre_cpu_update_frame(x, y);
                }
                (GameStateFrameData::Pause(_x), GameStateFrameData::Pause(_y)) => {
                    // pre_cpu_update_frame(x, y);
                }
                _ => panic!("unexpeced combination of states"),
            }
        }

        while accumulator >= dt {
            // update the game for a fixed number of steps
            accumulator -= dt;

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

            for i in (0..frame_params.gameplay_data.len()).rev() {
                next_game_state = match frame_params.gameplay_data.get_mut(i).unwrap() {
                    GameStateFrameData::Gameplay(x) => update_gameplay_state(x, &messages),
                    GameStateFrameData::Pause(x) => update_pause_state(x, &messages),
                };
            }

            update_frame_number += 1;
        }

        // draw the game
        let mut gpu_heap = LinearAllocator {
            gpu_data: map_gpu_buffer(
                &frame_params.cpu_render.frame_constant_buffer,
                &graphics_layer,
            ),
            state: LinearAllocatorState { used_bytes: 0 },
        };

        for i in 0..frame_params.gameplay_data.len() {
            match frame_params.gameplay_data.get_mut(i).unwrap() {
                GameStateFrameData::Gameplay(x) => draw_gameplay_state(
                    x,
                    &mut graphics_layer.graphics_command_list,
                    &graphics_layer.backbuffer_rtv,
                    &screenspace_quad_pso,
                    &gpu_heap.gpu_data,
                    &mut gpu_heap.state,
                ),

                GameStateFrameData::Pause(x) => draw_pause_state(
                    x,
                    &mut graphics_layer.graphics_command_list,
                    &graphics_layer.backbuffer_rtv,
                    &screenspace_quad_blended_pso,
                    &gpu_heap.gpu_data,
                    &mut gpu_heap.state,
                ),
            };
        }

        // unmap the gpu buffer
        // from this point onwards we are unable to allocate further memory
        unmap_gpu_buffer(gpu_heap.gpu_data, &graphics_layer);

        execute_command_list(&graphics_layer, &graphics_layer.graphics_command_list);

        present_swapchain(&graphics_layer);
    }
}
