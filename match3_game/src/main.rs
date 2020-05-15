use crate::gameplay::draw_gameplay_state;
use crate::gameplay::update_gameplay_state;
use crate::gameplay::GameplayState;
use crate::pause::draw_pause_state;
use crate::pause::update_pause_state;
use crate::pause::PauseState;
use graphics_device::*;
use os_window::*;

// this will cause pause.rs be included into this compilation unit
mod gameplay;
mod pause;

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

///  -------------------- gameplay ---------------------

pub enum GameStateType {
    //MainMenu,
    Pause,
    Gameplay,
}

enum GameStateData<'a> {
    Gameplay(GameplayState<'a>),
    Pause(PauseState<'a>),
}

// data for each displayed frame
// frame = "A piece of data that is processed and ultimately displayed on screen"
struct FrameParams {
    cpu_render: CpuRenderFrameData,
}

fn clamp<T: std::cmp::PartialOrd>(x: T, min: T, max: T) -> T {
    if x > max {
        max
    } else if x < min {
        min
    } else {
        x
    }
}

pub struct UpdateBehaviourDesc {
    // tells the system if a state trasition is required
    transition_state: GameStateTransitionState,

    // this allows a state to block all input from reaching lower level frames
    // could be extended so that only certain input values are blocked
    block_input: bool,
}

enum GameStateTransitionState {
    Unchanged,
    TransitionToNewState(GameStateType),
    ReturnToPreviousState,
}

fn main() {
    let args: CommandLineArgs = parse_cmdline();

    let mut should_game_close = false;

    // afterwards open a window we can render into
    let main_window: Window = create_window(540, 960).unwrap();

    let mut graphics_layer: GraphicsDeviceLayer =
        create_device_graphics_layer(main_window.hwnd, args.enable_debug_device).unwrap();

    let mut engine_frame_params0 = FrameParams {
        cpu_render: CpuRenderFrameData {
            frame_constant_buffer: create_constant_buffer(
                &graphics_layer,
                1024 * 8,
                "Frame 0 Constants",
            ),
        },
    };

    let mut engine_frame_params1 = FrameParams {
        cpu_render: CpuRenderFrameData {
            frame_constant_buffer: create_constant_buffer(
                &graphics_layer,
                1024 * 8,
                "Frame 1 Constants",
            ),
        },
    };

    let dt: f32 = 1.0 / 60.0;
    let mut accumulator: f32 = dt;

    let mut current_time = std::time::Instant::now();
    let mut update_frame_number: u64 = 0;

    let mut game_state_stack: Vec<GameStateData> = Vec::new();
    let mut next_game_state: GameStateTransitionState =
        GameStateTransitionState::TransitionToNewState(GameStateType::Gameplay);

    while !should_game_close {
        let new_time = std::time::Instant::now();

        // calculate how much time has passed
        let frame_time = f32::min(
            as_fractional_secs(&new_time.duration_since(current_time)),
            0.25,
        );

        accumulator += frame_time;

        current_time = new_time;

        // for now just sleep
        // don't want to waste CPU resources rendering more frames
        // this is a match3 game, 30fps will be fine
        if accumulator < dt {
            let sleep_duration = dt - accumulator;

            std::thread::sleep(std::time::Duration::from_secs_f32(sleep_duration));
        }

        accumulator = dt;

        // we are starting a new frame, do we need to transition to a new state?
        match next_game_state {
            GameStateTransitionState::TransitionToNewState(x) => {
                match x {
                    GameStateType::Gameplay => {
                        game_state_stack
                            .push(GameStateData::Gameplay(GameplayState::new(&graphics_layer)));
                    }

                    GameStateType::Pause => {
                        game_state_stack
                            .push(GameStateData::Pause(PauseState::new(&graphics_layer)));
                    }
                }

                // make sure to reset the state
                next_game_state = GameStateTransitionState::Unchanged;
            }

            GameStateTransitionState::ReturnToPreviousState => {
                // remove the top most state from the stack
                game_state_stack.pop();

                // close the game once all game states have been deleted
                if game_state_stack.len() == 0 {
                    should_game_close = true;
                    continue;
                }

                // make sure to reset the state
                next_game_state = GameStateTransitionState::Unchanged;
            }

            GameStateTransitionState::Unchanged => {}
        }

        let (_prev_engine_frame_params, engine_frame_params) = if update_frame_number % 2 == 0 {
            (&engine_frame_params1, &mut engine_frame_params0)
        } else {
            (&engine_frame_params0, &mut engine_frame_params1)
        };

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

            for state in game_state_stack.iter_mut().rev() {
                let state_status = match state {
                    GameStateData::Gameplay(game_state) => {
                        let (prev_frame_params, frame_params) = if update_frame_number % 2 == 0 {
                            (&game_state.frame_data0, &mut game_state.frame_data1)
                        } else {
                            (&game_state.frame_data1, &mut game_state.frame_data0)
                        };

                        update_gameplay_state(prev_frame_params, frame_params, &messages, dt)
                    }

                    GameStateData::Pause(game_state) => {
                        let (prev_frame_params, frame_params) = if update_frame_number % 2 == 0 {
                            (&game_state.frame_data0, &mut game_state.frame_data1)
                        } else {
                            (&game_state.frame_data1, &mut game_state.frame_data0)
                        };

                        update_pause_state(prev_frame_params, frame_params, &messages, dt)
                    }
                };

                if state_status.block_input {
                    messages.clear();
                }

                match state_status.transition_state {
                    GameStateTransitionState::Unchanged => {}
                    _ => match next_game_state {
                        GameStateTransitionState::Unchanged => {
                            next_game_state = state_status.transition_state;
                        }
                        _ => {
                            panic!("logic error, only one state transition per frame is allowed");
                        }
                    },
                }
            }

            update_frame_number += 1;
        }

        // draw the game
        let mut gpu_heap = LinearAllocator {
            gpu_data: map_gpu_buffer(
                &engine_frame_params.cpu_render.frame_constant_buffer,
                &graphics_layer,
            ),
            state: LinearAllocatorState { used_bytes: 0 },
        };

        for state in game_state_stack.iter_mut() {
            match state {
                GameStateData::Gameplay(game_state) => {
                    let frame_params = if update_frame_number % 2 == 0 {
                        &game_state.frame_data1
                    } else {
                        &game_state.frame_data0
                    };

                    draw_gameplay_state(
                        &game_state.static_data,
                        frame_params,
                        &mut graphics_layer.graphics_command_list,
                        &graphics_layer.backbuffer_rtv,
                        &gpu_heap.gpu_data,
                        &mut gpu_heap.state,
                    );
                }

                GameStateData::Pause(x) => {
                    let frame_params = if update_frame_number % 2 == 0 {
                        &x.frame_data1
                    } else {
                        &x.frame_data0
                    };

                    draw_pause_state(
                        &x.static_data,
                        frame_params,
                        &mut graphics_layer.graphics_command_list,
                        &graphics_layer.backbuffer_rtv,
                        &gpu_heap.gpu_data,
                        &mut gpu_heap.state,
                    )
                }
            };
        }

        // unmap the gpu buffer
        // from this point onwards we are unable to allocate further memory
        unmap_gpu_buffer(gpu_heap.gpu_data, &graphics_layer);

        execute_command_list(&graphics_layer, &graphics_layer.graphics_command_list);

        present_swapchain(&graphics_layer);
    }
}
