use crate::entity::direction::Direction;

#[derive(Debug, Clone, Copy)]
pub enum GameCommand {
    MovePlayer(Direction),
    Exit,
    TogglePause,
    ToggleDebug,
    MuteAudio,
    ResetLevel,
}
