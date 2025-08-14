use crate::input::commands::GameCommand;

#[derive(Debug, Clone, Copy)]
pub enum GameEvent {
    InputCommand(GameCommand),
}

impl From<GameCommand> for GameEvent {
    fn from(command: GameCommand) -> Self {
        GameEvent::InputCommand(command)
    }
}
