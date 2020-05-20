use crate::gamestates::gameplay::{update_gameplay_state, GameplayState};
use crate::gamestates::pause::{update_pause_state, PauseState};
use graphics_device::GraphicsDeviceLayer;
use os_window::WindowMessages;

pub struct UpdateBehaviourDesc {
    // tells the system if a state trasition is required
    pub transition_state: GameStateTransitionState,

    // this allows a state to block all input from reaching lower level frames
    // could be extended so that only certain input values are blocked
    pub block_input: bool,
}

pub mod gameplay;
pub mod pause;

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

pub fn execute_possible_state_transition(
    state_transition: GameStateTransitionState,
    game_state_stack: &mut Vec<GameStateData>,
    graphics_layer: &GraphicsDeviceLayer,
) {
    // we are starting a new frame, do we need to transition to a new state?
    match state_transition {
        GameStateTransitionState::TransitionToNewState(x) => match x {
            GameStateType::Gameplay => {
                game_state_stack.push(GameStateData::Gameplay(GameplayState::new(&graphics_layer)));
            }

            GameStateType::Pause => {
                game_state_stack.push(GameStateData::Pause(PauseState::new(&graphics_layer)));
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
