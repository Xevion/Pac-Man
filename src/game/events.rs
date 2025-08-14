use bevy_ecs::event::Event;

use crate::input::commands::GameCommand;

#[derive(Debug, Clone, Copy, Event)]
pub enum GameEvent {
    Command(GameCommand),
}

impl From<GameCommand> for GameEvent {
    fn from(command: GameCommand) -> Self {
        GameEvent::Command(command)
    }
}
