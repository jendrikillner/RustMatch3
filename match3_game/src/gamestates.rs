mod gameplay;
mod pause;

use crate::gamestates::gameplay::draw_gameplay_state;
use crate::gamestates::gameplay::{update_gameplay_state, GameplayState};
use crate::gamestates::pause::draw_pause_state;
use crate::gamestates::pause::{update_pause_state, PauseState};
use graphics_device::GraphicsCommandList;
use graphics_device::GraphicsDevice;
use graphics_device::LinearAllocatorState;
use graphics_device::MappedGpuData;
use graphics_device::RenderTargetView;
use os_window::WindowMessages;

pub struct UpdateBehaviourDesc {
    // tells the system if a state trasition is required
    pub transition_state: GameStateTransitionState,

    // this allows a state to block all input from reaching lower level frames
    // could be extended so that only certain input values are blocked
    pub block_input: bool,
}

pub enum GameStateType {
    //MainMenu,
    Pause,
    Gameplay,
}

pub enum GameStateData<'a> {
    Gameplay(GameplayState<'a>),
    Pause(PauseState<'a>),
}

pub enum GameStateTransitionState {
    Unchanged,
    TransitionToNewState(GameStateType),
    ReturnToPreviousState,
}

pub fn execute_possible_state_transition<'a>(
    state_transition: GameStateTransitionState,
    game_state_stack: &mut Vec<GameStateData<'a>>,
    graphics_device: &'a GraphicsDevice,
) {
    // we are starting a new frame, do we need to transition to a new state?
    match state_transition {
        GameStateTransitionState::TransitionToNewState(x) => match x {
            GameStateType::Gameplay => {
                game_state_stack.push(GameStateData::Gameplay(GameplayState::new(graphics_device)));
            }

            GameStateType::Pause => {
                game_state_stack.push(GameStateData::Pause(PauseState::new(graphics_device)));
            }
        },

        GameStateTransitionState::ReturnToPreviousState => {
            // remove the top most state from the stack
            game_state_stack.pop();
        }

        GameStateTransitionState::Unchanged => {}
    }
}

pub fn update_gamestate_stack(
    dt: f32,
    update_frame_number: u64,
    game_state_stack: &mut Vec<GameStateData>,
    messages: &mut Vec<WindowMessages>,
) -> GameStateTransitionState {
    let mut game_state_transtion = GameStateTransitionState::Unchanged;

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
            _ => match game_state_transtion {
                GameStateTransitionState::Unchanged => {
                    game_state_transtion = state_status.transition_state;
                }
                _ => {
                    panic!("logic error, only one state transition per frame is allowed");
                }
            },
        }
    }

    game_state_transtion
}

pub fn draw_gamestate_stack(
    game_state_stack: &[GameStateData],
    frame_number: u64,
    command_list: &mut GraphicsCommandList,
    backbuffer_rtv: &RenderTargetView,
    gpu_heap_data: &MappedGpuData,
    gpu_heap_state: &mut LinearAllocatorState,
) {
    for state in game_state_stack.iter() {
        match state {
            GameStateData::Gameplay(game_state) => {
                let frame_params = if frame_number % 2 == 0 {
                    &game_state.frame_data1
                } else {
                    &game_state.frame_data0
                };

                draw_gameplay_state(
                    &game_state.static_data,
                    frame_params,
                    command_list,
                    backbuffer_rtv,
                    gpu_heap_data,
                    gpu_heap_state,
                );
            }

            GameStateData::Pause(x) => {
                let frame_params = if frame_number % 2 == 0 {
                    &x.frame_data1
                } else {
                    &x.frame_data0
                };

                draw_pause_state(
                    &x.static_data,
                    frame_params,
                    command_list,
                    backbuffer_rtv,
                    gpu_heap_data,
                    gpu_heap_state,
                )
            }
        };
    }
}
