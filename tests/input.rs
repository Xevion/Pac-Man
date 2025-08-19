use pacman::events::{GameCommand, GameEvent};
use pacman::map::direction::Direction;
use pacman::systems::input::{process_simple_key_events, Bindings, SimpleKeyEvent};
use sdl2::keyboard::Keycode;

#[test]
fn resumes_previous_direction_when_secondary_key_released() {
    let mut bindings = Bindings::default();

    // Frame 1: Press W (Up) => emits Move Up
    let events = process_simple_key_events(&mut bindings, &[SimpleKeyEvent::KeyDown(Keycode::W)]);
    assert!(events.contains(&GameEvent::Command(GameCommand::MovePlayer(Direction::Up))));

    // Frame 2: Press D (Right) => emits Move Right
    let events = process_simple_key_events(&mut bindings, &[SimpleKeyEvent::KeyDown(Keycode::D)]);
    assert!(events.contains(&GameEvent::Command(GameCommand::MovePlayer(Direction::Right))));

    // Frame 3: Release D, no new key this frame => should continue previous key W (Up)
    let events = process_simple_key_events(&mut bindings, &[SimpleKeyEvent::KeyUp(Keycode::D)]);
    assert!(events.contains(&GameEvent::Command(GameCommand::MovePlayer(Direction::Up))));
}

#[test]
fn holds_last_pressed_key_across_frames_when_no_new_input() {
    let mut bindings = Bindings::default();

    // Frame 1: Press Left
    let events = process_simple_key_events(&mut bindings, &[SimpleKeyEvent::KeyDown(Keycode::Left)]);
    assert!(events.contains(&GameEvent::Command(GameCommand::MovePlayer(Direction::Left))));

    // Frame 2: No input => continues Left
    let events = process_simple_key_events(&mut bindings, &[]);
    assert!(events.contains(&GameEvent::Command(GameCommand::MovePlayer(Direction::Left))));

    // Frame 3: Release Left, no input remains => nothing emitted
    let events = process_simple_key_events(&mut bindings, &[SimpleKeyEvent::KeyUp(Keycode::Left)]);
    assert!(events.is_empty());
}
