use glam::Vec2;
use pacman::events::{GameCommand, GameEvent};
use pacman::map::direction::Direction;
use pacman::systems::input::{
    calculate_direction_from_delta, process_simple_key_events, update_touch_reference_position, Bindings, CursorPosition,
    SimpleKeyEvent, TouchData, TouchState, TOUCH_DIRECTION_THRESHOLD, TOUCH_EASING_DISTANCE_THRESHOLD,
};
use sdl2::keyboard::Keycode;
use speculoos::prelude::*;

// Test modules for better organization
mod keyboard_tests {
    use super::*;

    #[test]
    fn key_down_emits_bound_command() {
        let mut bindings = Bindings::default();
        let events = process_simple_key_events(&mut bindings, &[SimpleKeyEvent::KeyDown(Keycode::W)]);
        assert_that(&events).contains(GameEvent::Command(GameCommand::MovePlayer(Direction::Up)));
    }

    #[test]
    fn key_down_emits_non_movement_commands() {
        let mut bindings = Bindings::default();
        let events = process_simple_key_events(&mut bindings, &[SimpleKeyEvent::KeyDown(Keycode::Escape)]);
        assert_that(&events).contains(GameEvent::Command(GameCommand::TogglePause));
    }

    #[test]
    fn unbound_key_emits_nothing() {
        let mut bindings = Bindings::default();
        let events = process_simple_key_events(&mut bindings, &[SimpleKeyEvent::KeyDown(Keycode::Z)]);
        assert_that(&events).is_empty();
    }

    #[test]
    fn movement_key_held_continues_across_frames() {
        let mut bindings = Bindings::default();
        process_simple_key_events(&mut bindings, &[SimpleKeyEvent::KeyDown(Keycode::Left)]);
        let events = process_simple_key_events(&mut bindings, &[]);
        assert_that(&events).contains(GameEvent::Command(GameCommand::MovePlayer(Direction::Left)));
    }

    #[test]
    fn releasing_movement_key_stops_continuation() {
        let mut bindings = Bindings::default();
        process_simple_key_events(&mut bindings, &[SimpleKeyEvent::KeyDown(Keycode::Up)]);
        let events = process_simple_key_events(&mut bindings, &[SimpleKeyEvent::KeyUp(Keycode::Up)]);
        assert_that(&events).is_empty();
    }

    #[test]
    fn multiple_movement_keys_resumes_previous_when_current_released() {
        let mut bindings = Bindings::default();
        process_simple_key_events(&mut bindings, &[SimpleKeyEvent::KeyDown(Keycode::W)]);
        process_simple_key_events(&mut bindings, &[SimpleKeyEvent::KeyDown(Keycode::D)]);
        let events = process_simple_key_events(&mut bindings, &[SimpleKeyEvent::KeyUp(Keycode::D)]);
        assert_that(&events).contains(GameEvent::Command(GameCommand::MovePlayer(Direction::Up)));
    }
}

mod direction_calculation_tests {
    use super::*;

    #[test]
    fn prioritizes_horizontal_movement() {
        let test_cases = vec![
            (Vec2::new(6.0, 5.0), Direction::Right),
            (Vec2::new(-6.0, 5.0), Direction::Left),
        ];

        for (delta, expected) in test_cases {
            assert_that(&calculate_direction_from_delta(delta)).is_equal_to(expected);
        }
    }

    #[test]
    fn uses_vertical_when_dominant() {
        let test_cases = vec![
            (Vec2::new(3.0, 10.0), Direction::Down),
            (Vec2::new(3.0, -10.0), Direction::Up),
        ];

        for (delta, expected) in test_cases {
            assert_that(&calculate_direction_from_delta(delta)).is_equal_to(expected);
        }
    }

    #[test]
    fn handles_zero_delta() {
        let delta = Vec2::ZERO;
        // Should default to Up when both components are zero
        assert_that(&calculate_direction_from_delta(delta)).is_equal_to(Direction::Up);
    }

    #[test]
    fn handles_equal_magnitudes() {
        // When x and y have equal absolute values, should prioritize vertical
        let delta = Vec2::new(5.0, 5.0);
        assert_that(&calculate_direction_from_delta(delta)).is_equal_to(Direction::Down);

        let delta = Vec2::new(-5.0, 5.0);
        assert_that(&calculate_direction_from_delta(delta)).is_equal_to(Direction::Down);
    }
}

mod touch_easing_tests {
    use super::*;

    #[test]
    fn easing_within_threshold_does_nothing() {
        let mut touch_data = TouchData::new(0, Vec2::new(100.0, 100.0));
        touch_data.current_pos = Vec2::new(100.0 + TOUCH_EASING_DISTANCE_THRESHOLD - 0.1, 100.0);

        let (_delta, distance) = update_touch_reference_position(&mut touch_data, 0.016);

        assert_that(&distance).is_less_than(TOUCH_EASING_DISTANCE_THRESHOLD);
        assert_that(&touch_data.start_pos).is_equal_to(Vec2::new(100.0, 100.0));
    }

    #[test]
    fn easing_beyond_threshold_moves_towards_target() {
        let mut touch_data = TouchData::new(0, Vec2::new(100.0, 100.0));
        touch_data.current_pos = Vec2::new(150.0, 100.0);

        let original_start_pos = touch_data.start_pos;
        let (_delta, distance) = update_touch_reference_position(&mut touch_data, 0.016);

        assert_that(&distance).is_greater_than(TOUCH_EASING_DISTANCE_THRESHOLD);
        assert_that(&touch_data.start_pos.x).is_greater_than(original_start_pos.x);
        assert_that(&touch_data.start_pos.x).is_less_than(touch_data.current_pos.x);
    }

    #[test]
    fn easing_overshoot_sets_to_target() {
        let mut touch_data = TouchData::new(0, Vec2::new(100.0, 100.0));
        touch_data.current_pos = Vec2::new(101.0, 100.0);

        let (_delta, _distance) = update_touch_reference_position(&mut touch_data, 10.0);

        assert_that(&touch_data.start_pos).is_equal_to(touch_data.current_pos);
    }

    #[test]
    fn easing_returns_correct_delta() {
        let mut touch_data = TouchData::new(0, Vec2::new(100.0, 100.0));
        touch_data.current_pos = Vec2::new(120.0, 110.0);

        let (delta, distance) = update_touch_reference_position(&mut touch_data, 0.016);

        let expected_delta = Vec2::new(20.0, 10.0);
        let expected_distance = expected_delta.length();

        assert_that(&delta).is_equal_to(expected_delta);
        assert_that(&distance).is_equal_to(expected_distance);
    }
}

// Integration tests for the full input system
mod integration_tests {
    use super::*;

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
        use bevy_ecs::{event::Events, system::RunSystemOnce, world::World};
        use pacman::systems::components::DeltaTime;
        use pacman::systems::input::input_system;

        let sdl_context = sdl2::init().expect("Failed to initialize SDL");
        let event_subsystem = sdl_context.event().expect("Failed to get event subsystem");
        let event_pump = sdl_context.event_pump().expect("Failed to create event pump");

        let mut world = World::new();
        world.insert_resource(Events::<GameEvent>::default());
        world.insert_resource(DeltaTime {
            seconds: delta_time,
            ticks: 1,
        });
        world.insert_resource(Bindings::default());
        world.insert_resource(CursorPosition::None);
        world.insert_resource(TouchState::default());
        world.insert_non_send_resource(event_pump);

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
                assert_that(&position).is_equal_to(Vec2::new(100.0, 200.0));
                assert_that(&remaining_time).is_equal_to(0.20);
            }
            CursorPosition::None => panic!("Expected cursor position to be set"),
        }
    }

    #[test]
    fn mouse_button_down_starts_touch() {
        let events = vec![mouse_button_down_event(150, 250)];
        let (_cursor, touch_state) = run_input_system_with_events(events, 0.016);

        assert_that(&touch_state.active_touch).is_some();
        if let Some(touch_data) = &touch_state.active_touch {
            assert_that(&touch_data.finger_id).is_equal_to(0);
            assert_that(&touch_data.start_pos).is_equal_to(Vec2::new(150.0, 250.0));
        }
    }

    #[test]
    fn mouse_button_up_ends_touch() {
        let events = vec![mouse_button_down_event(150, 250), mouse_button_up_event(150, 250)];
        let (_cursor, touch_state) = run_input_system_with_events(events, 0.016);

        assert_that(&touch_state.active_touch).is_none();
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

        assert_that(&distance).is_greater_than_or_equal_to(TOUCH_DIRECTION_THRESHOLD);
        let direction = calculate_direction_from_delta(delta);
        assert_that(&direction).is_equal_to(Direction::Right);
    }

    #[test]
    fn movement_below_threshold_no_direction() {
        let mut touch_data = TouchData::new(1, Vec2::new(100.0, 100.0));
        touch_data.current_pos = Vec2::new(100.0 + TOUCH_DIRECTION_THRESHOLD - 1.0, 100.0);

        let (_delta, distance) = update_touch_reference_position(&mut touch_data, 0.016);

        assert_that(&distance).is_less_than(TOUCH_DIRECTION_THRESHOLD);
    }

    #[test]
    fn all_directions_work_correctly() {
        let test_cases = vec![
            (Vec2::new(TOUCH_DIRECTION_THRESHOLD + 5.0, 0.0), Direction::Right),
            (Vec2::new(-TOUCH_DIRECTION_THRESHOLD - 5.0, 0.0), Direction::Left),
            (Vec2::new(0.0, TOUCH_DIRECTION_THRESHOLD + 5.0), Direction::Down),
            (Vec2::new(0.0, -TOUCH_DIRECTION_THRESHOLD - 5.0), Direction::Up),
        ];

        for (offset, expected_direction) in test_cases {
            let mut touch_data = TouchData::new(1, Vec2::new(100.0, 100.0));
            touch_data.current_pos = Vec2::new(100.0, 100.0) + offset;

            let (delta, distance) = update_touch_reference_position(&mut touch_data, 0.016);

            assert_that(&distance).is_greater_than_or_equal_to(TOUCH_DIRECTION_THRESHOLD);
            let direction = calculate_direction_from_delta(delta);
            assert_that(&direction).is_equal_to(expected_direction);
        }
    }
}
