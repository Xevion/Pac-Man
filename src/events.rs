use bevy_ecs::{entity::Entity, event::Event};

use crate::map::direction::Direction;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameCommand {
    Exit,
    MovePlayer(Direction),
    ToggleDebug,
    MuteAudio,
    ResetLevel,
    TogglePause,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameEvent {
    Command(GameCommand),
    Collision(Entity, Entity),
}

impl From<GameCommand> for GameEvent {
    fn from(command: GameCommand) -> Self {
        GameEvent::Command(command)
    }
}
