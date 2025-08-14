use crate::entity::direction::Direction;

#[derive(Debug, Clone, Copy)]
pub enum GameCommand {
    MovePlayer(Direction),
    TogglePause,
    ToggleDebug,
    MuteAudio,
    ResetLevel,
    Exit,
}
