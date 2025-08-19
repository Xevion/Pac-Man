use pacman::events::{GameCommand, GameEvent};
use pacman::map::direction::Direction;









#[test]
fn test_game_command_to_game_event_conversion_all_variants() {
    let commands = vec![
        GameCommand::Exit,
        GameCommand::MovePlayer(Direction::Up),
        GameCommand::ToggleDebug,
        GameCommand::MuteAudio,
        GameCommand::ResetLevel,
        GameCommand::TogglePause,
    ];

    for command in commands {
        let event: GameEvent = command.into();
        assert_eq!(event, GameEvent::Command(command));
    }
}






