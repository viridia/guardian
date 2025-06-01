use bevy::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
pub enum GameState {
    Intro,
    // TODO: Switch default to intro later.
    #[default]
    Playing,
    LevelComplete,
}

#[derive(SubStates, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[source(GameState = GameState::Playing)]
pub enum PauseState {
    #[default]
    Running,
    Paused,
    GameOver,
}
