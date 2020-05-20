use crate::gamestates::gameplay::GameplayState;
use crate::gamestates::pause::PauseState;

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
