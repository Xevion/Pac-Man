use assert2::check;
use glam::Vec2;
use pacman::events::{GameCommand, GameEvent};
use pacman::map::direction::Direction;
use pacman::systems::input::{
    Bindings, CursorPosition, SimpleKeyEvent, TOUCH_DIRECTION_THRESHOLD, TOUCH_EASING_DISTANCE_THRESHOLD, TouchData, TouchState,
    calculate_direction_from_delta, process_simple_key_events, update_touch_reference_position,
};
use rstest::rstest;
use sdl2::keyboard::Keycode;

// Test modules for better organization
mod keyboard_tests {
    use super::*;

    #[test]
    fn key_down_emits_bound_command() {
        let mut bindings = Bindings::default();
        let events = process_simple_key_events(&mut bindings, &[SimpleKeyEvent::KeyDown(Keycode::W)]);
        check!(events.contains(&GameEvent::Command(GameCommand::MovePlayer(Direction::Up))));
    }

    #[test]
    fn key_down_emits_non_movement_commands() {
        let mut bindings = Bindings::default();
        let events = process_simple_key_events(&mut bindings, &[SimpleKeyEvent::KeyDown(Keycode::Escape)]);
        check!(events.contains(&GameEvent::Command(GameCommand::TogglePause)));
    }

    #[test]
    fn unbound_key_emits_nothing() {
        let mut bindings = Bindings::default();
        let events = process_simple_key_events(&mut bindings, &[SimpleKeyEvent::KeyDown(Keycode::Z)]);
        check!(events.is_empty());
    }

    #[test]
    fn movement_key_held_continues_across_frames() {
        let mut bindings = Bindings::default();
        process_simple_key_events(&mut bindings, &[SimpleKeyEvent::KeyDown(Keycode::Left)]);
        let events = process_simple_key_events(&mut bindings, &[]);
        check!(events.contains(&GameEvent::Command(GameCommand::MovePlayer(Direction::Left))));
    }

    #[test]
    fn releasing_movement_key_stops_continuation() {
        let mut bindings = Bindings::default();
        process_simple_key_events(&mut bindings, &[SimpleKeyEvent::KeyDown(Keycode::Up)]);
        let events = process_simple_key_events(&mut bindings, &[SimpleKeyEvent::KeyUp(Keycode::Up)]);
        check!(events.is_empty());
    }

    #[test]
    fn multiple_movement_keys_resumes_previous_when_current_released() {
        let mut bindings = Bindings::default();
        process_simple_key_events(&mut bindings, &[SimpleKeyEvent::KeyDown(Keycode::W)]);
        process_simple_key_events(&mut bindings, &[SimpleKeyEvent::KeyDown(Keycode::D)]);
        let events = process_simple_key_events(&mut bindings, &[SimpleKeyEvent::KeyUp(Keycode::D)]);
        check!(events.contains(&GameEvent::Command(GameCommand::MovePlayer(Direction::Up))));
    }
}

mod direction_calculation_tests {
    use super::*;

    #[rstest]
    #[case(Vec2::new(6.0, 5.0), Direction::Right)]
    #[case(Vec2::new(-6.0, 5.0), Direction::Left)]
    fn prioritizes_horizontal_movement(#[case] delta: Vec2, #[case] expected: Direction) {
        check!(calculate_direction_from_delta(delta) == expected);
    }

    #[rstest]
    #[case(Vec2::new(3.0, 10.0), Direction::Down)]
    #[case(Vec2::new(3.0, -10.0), Direction::Up)]
    fn uses_vertical_when_dominant(#[case] delta: Vec2, #[case] expected: Direction) {
        check!(calculate_direction_from_delta(delta) == expected);
    }

    #[test]
    fn handles_zero_delta() {
        let delta = Vec2::ZERO;
        // Should default to Up when both components are zero
        check!(calculate_direction_from_delta(delta) == Direction::Up);
    }

    #[test]
    fn handles_equal_magnitudes() {
        // When x and y have equal absolute values, should prioritize vertical
        let delta = Vec2::new(5.0, 5.0);
        check!(calculate_direction_from_delta(delta) == Direction::Down);

        let delta = Vec2::new(-5.0, 5.0);
        check!(calculate_direction_from_delta(delta) == Direction::Down);
    }
}

mod touch_easing_tests {
    use super::*;

    #[test]
    fn easing_within_threshold_does_nothing() {
        let mut touch_data = TouchData::new(0, Vec2::new(100.0, 100.0));
        touch_data.current_pos = Vec2::new(100.0 + TOUCH_EASING_DISTANCE_THRESHOLD - 0.1, 100.0);

        let (_delta, distance) = update_touch_reference_position(&mut touch_data, 0.016);

        check!(distance < TOUCH_EASING_DISTANCE_THRESHOLD);
        check!(touch_data.start_pos == Vec2::new(100.0, 100.0));
    }

    #[test]
    fn easing_beyond_threshold_moves_towards_target() {
        let mut touch_data = TouchData::new(0, Vec2::new(100.0, 100.0));
        touch_data.current_pos = Vec2::new(150.0, 100.0);

        let original_start_pos = touch_data.start_pos;
        let (_delta, distance) = update_touch_reference_position(&mut touch_data, 0.016);

        check!(distance > TOUCH_EASING_DISTANCE_THRESHOLD);
        check!(touch_data.start_pos.x > original_start_pos.x);
        check!(touch_data.start_pos.x < touch_data.current_pos.x);
    }

    #[test]
    fn easing_overshoot_sets_to_target() {
        let mut touch_data = TouchData::new(0, Vec2::new(100.0, 100.0));
        touch_data.current_pos = Vec2::new(101.0, 100.0);

        let (_delta, _distance) = update_touch_reference_position(&mut touch_data, 10.0);

        check!(touch_data.start_pos == touch_data.current_pos);
    }

    #[test]
    fn easing_returns_correct_delta() {
        let mut touch_data = TouchData::new(0, Vec2::new(100.0, 100.0));
        touch_data.current_pos = Vec2::new(120.0, 110.0);

        let (delta, distance) = update_touch_reference_position(&mut touch_data, 0.016);

        let expected_delta = Vec2::new(20.0, 10.0);
        let expected_distance = expected_delta.length();

        check!(delta == expected_delta);
        check!(distance == expected_distance);
    }
}

// Integration tests for the full input system
mod integration_tests {
    use super::*;
    use assert2::assert;

    fn mouse_motion_event(x: i32, y: i32) -> sdl2::event::Event {
        sdl2::event::Event::MouseMotion {
            x,
            y,
            xrel: 0,
            yrel: 0,
            mousestate: sdl2::mouse::MouseState::from_sdl_state(0),
            which: 0,
            window_id: 0,
            timestamp: 0,
        }
    }

    fn mouse_button_down_event(x: i32, y: i32) -> sdl2::event::Event {
        sdl2::event::Event::MouseButtonDown {
            x,
            y,
            mouse_btn: sdl2::mouse::MouseButton::Left,
            clicks: 1,
            which: 0,
            window_id: 0,
            timestamp: 0,
        }
    }

    fn mouse_button_up_event(x: i32, y: i32) -> sdl2::event::Event {
        sdl2::event::Event::MouseButtonUp {
            x,
            y,
            mouse_btn: sdl2::mouse::MouseButton::Left,
            clicks: 1,
            which: 0,
            window_id: 0,
            timestamp: 0,
        }
    }

    // Simplified helper for testing SDL integration
    fn run_input_system_with_events(events: Vec<sdl2::event::Event>, delta_time: f32) -> (CursorPosition, TouchState) {
        use bevy_ecs::{message::Messages, system::RunSystemOnce, world::World};
        use pacman::systems::common::DeltaTime;
        use pacman::systems::input::{HumanInput, InputSource, input_system};
        use pacman::systems::layout::{Layout, Orientation};

        let sdl_context = sdl2::init().expect("Failed to initialize SDL");
        let event_subsystem = sdl_context.event().expect("Failed to get event subsystem");
        let event_pump = sdl_context.event_pump().expect("Failed to create event pump");

        let mut world = World::new();
        world.insert_resource(Messages::<GameEvent>::default());
        world.insert_resource(DeltaTime {
            seconds: delta_time,
            ticks: 1,
        });
        world.insert_resource(Bindings::default());
        world.insert_resource(InputSource::default());
        world.insert_resource(HumanInput::default());
        world.insert_resource(CursorPosition::None);
        world.insert_resource(TouchState::default());
        // Identity layout (maze at the origin, scale 1) so window->maze mapping is a
        // no-op; the layout math itself is covered by the layout unit tests.
        world.insert_resource(Layout {
            window: glam::UVec2::new(224, 248),
            scale: 1,
            orientation: Orientation::Portrait,
            maze: sdl2::rect::Rect::new(0, 0, 224, 248),
            left: None,
            right: None,
            top: None,
            bottom: None,
        });
        world.insert_non_send(event_pump);

        // Inject events into SDL's event queue
        for event in events {
            event_subsystem.push_event(event).expect("Failed to push event");
        }

        // Run the real input system
        world
            .run_system_once(input_system)
            .expect("Input system should run successfully");

        let cursor = *world.resource::<CursorPosition>();
        let touch_state = world.resource::<TouchState>().clone();

        (cursor, touch_state)
    }

    #[test]
    fn mouse_motion_updates_cursor_position() {
        let events = vec![mouse_motion_event(100, 200)];
        let (cursor, _touch_state) = run_input_system_with_events(events, 0.016);

        match cursor {
            CursorPosition::Some {
                position,
                remaining_time,
            } => {
                check!(position == Vec2::new(100.0, 200.0));
                check!(remaining_time == 0.20);
            }
            CursorPosition::None => panic!("Expected cursor position to be set"),
        }
    }

    #[test]
    fn mouse_button_down_starts_touch() {
        let events = vec![mouse_button_down_event(150, 250)];
        let (_cursor, touch_state) = run_input_system_with_events(events, 0.016);

        assert!(let Some(touch_data) = &touch_state.active_touch);
        check!(touch_data.finger_id == 0);
        check!(touch_data.start_pos == Vec2::new(150.0, 250.0));
    }

    #[test]
    fn mouse_button_up_ends_touch() {
        let events = vec![mouse_button_down_event(150, 250), mouse_button_up_event(150, 250)];
        let (_cursor, touch_state) = run_input_system_with_events(events, 0.016);

        check!(touch_state.active_touch.is_none());
    }
}

// Touch direction tests
mod touch_direction_tests {
    use super::*;

    #[test]
    fn movement_above_threshold_emits_direction() {
        let mut touch_data = TouchData::new(1, Vec2::new(100.0, 100.0));
        touch_data.current_pos = Vec2::new(100.0 + TOUCH_DIRECTION_THRESHOLD + 5.0, 100.0);

        let (delta, distance) = update_touch_reference_position(&mut touch_data, 0.016);

        check!(distance >= TOUCH_DIRECTION_THRESHOLD);
        let direction = calculate_direction_from_delta(delta);
        check!(direction == Direction::Right);
    }

    #[test]
    fn movement_below_threshold_no_direction() {
        let mut touch_data = TouchData::new(1, Vec2::new(100.0, 100.0));
        touch_data.current_pos = Vec2::new(100.0 + TOUCH_DIRECTION_THRESHOLD - 1.0, 100.0);

        let (_delta, distance) = update_touch_reference_position(&mut touch_data, 0.016);

        check!(distance < TOUCH_DIRECTION_THRESHOLD);
    }

    #[rstest]
    #[case(Vec2::new(TOUCH_DIRECTION_THRESHOLD + 5.0, 0.0), Direction::Right)]
    #[case(Vec2::new(-TOUCH_DIRECTION_THRESHOLD - 5.0, 0.0), Direction::Left)]
    #[case(Vec2::new(0.0, TOUCH_DIRECTION_THRESHOLD + 5.0), Direction::Down)]
    #[case(Vec2::new(0.0, -TOUCH_DIRECTION_THRESHOLD - 5.0), Direction::Up)]
    fn all_directions_work_correctly(#[case] offset: Vec2, #[case] expected_direction: Direction) {
        let mut touch_data = TouchData::new(1, Vec2::new(100.0, 100.0));
        touch_data.current_pos = Vec2::new(100.0, 100.0) + offset;

        let (delta, distance) = update_touch_reference_position(&mut touch_data, 0.016);

        check!(distance >= TOUCH_DIRECTION_THRESHOLD);
        let direction = calculate_direction_from_delta(delta);
        check!(direction == expected_direction);
    }
}
