use pacman::events::{GameCommand, GameEvent};
use pacman::map::direction::Direction;

#[test]
fn test_game_command_variants() {
    // Test that all GameCommand variants can be created
    let commands = [
        GameCommand::Exit,
        GameCommand::MovePlayer(Direction::Up),
        GameCommand::MovePlayer(Direction::Down),
        GameCommand::MovePlayer(Direction::Left),
        GameCommand::MovePlayer(Direction::Right),
        GameCommand::ToggleDebug,
        GameCommand::MuteAudio,
        GameCommand::ResetLevel,
        GameCommand::TogglePause,
    ];

    // Just verify they can be created and compared
    assert_eq!(commands.len(), 9);
    assert_eq!(commands[0], GameCommand::Exit);
    assert_eq!(commands[1], GameCommand::MovePlayer(Direction::Up));
}

#[test]
fn test_game_command_equality() {
    assert_eq!(GameCommand::Exit, GameCommand::Exit);
    assert_eq!(GameCommand::ToggleDebug, GameCommand::ToggleDebug);
    assert_eq!(
        GameCommand::MovePlayer(Direction::Left),
        GameCommand::MovePlayer(Direction::Left)
    );

    assert_ne!(GameCommand::Exit, GameCommand::ToggleDebug);
    assert_ne!(
        GameCommand::MovePlayer(Direction::Left),
        GameCommand::MovePlayer(Direction::Right)
    );
}

#[test]
fn test_game_event_variants() {
    let command_event = GameEvent::Command(GameCommand::Exit);
    let collision_event = GameEvent::Collision(bevy_ecs::entity::Entity::from_raw(1), bevy_ecs::entity::Entity::from_raw(2));

    // Test that events can be created and compared
    assert_eq!(command_event, GameEvent::Command(GameCommand::Exit));
    assert_ne!(command_event, collision_event);
}

#[test]
fn test_game_command_to_game_event_conversion() {
    let command = GameCommand::ToggleDebug;
    let event: GameEvent = command.into();

    assert_eq!(event, GameEvent::Command(GameCommand::ToggleDebug));
}

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

#[test]
fn test_move_player_all_directions() {
    let directions = [Direction::Up, Direction::Down, Direction::Left, Direction::Right];

    for direction in directions {
        let command = GameCommand::MovePlayer(direction);
        let event: GameEvent = command.into();

        if let GameEvent::Command(GameCommand::MovePlayer(dir)) = event {
            assert_eq!(dir, direction);
        } else {
            panic!("Expected MovePlayer command with direction {:?}", direction);
        }
    }
}

#[test]
fn test_game_event_debug_format() {
    let event = GameEvent::Command(GameCommand::Exit);
    let debug_str = format!("{:?}", event);
    assert!(debug_str.contains("Command"));
    assert!(debug_str.contains("Exit"));
}

#[test]
fn test_game_command_debug_format() {
    let command = GameCommand::MovePlayer(Direction::Left);
    let debug_str = format!("{:?}", command);
    assert!(debug_str.contains("MovePlayer"));
    assert!(debug_str.contains("Left"));
}
