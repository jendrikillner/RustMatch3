use crate::gamestates::gameplay::GameplayState;
use crate::gamestates::pause::PauseState;
use graphics_device::GraphicsDeviceLayer;

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
