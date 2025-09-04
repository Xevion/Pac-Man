use glam::Vec2;
use pacman::map::direction::Direction;
use pacman::systems::movement::{BufferedDirection, Position, Velocity};

mod common;

#[test]
fn test_position_is_at_node() {
    let stopped_pos = Position::Stopped { node: 0 };
    let moving_pos = Position::Moving {
        from: 0,
        to: 1,
        remaining_distance: 8.0,
    };

    assert!(stopped_pos.is_at_node());
    assert!(!moving_pos.is_at_node());
}

#[test]
fn test_position_current_node() {
    let stopped_pos = Position::Stopped { node: 5 };
    let moving_pos = Position::Moving {
        from: 3,
        to: 7,
        remaining_distance: 12.0,
    };

    assert_eq!(stopped_pos.current_node(), 5);
    assert_eq!(moving_pos.current_node(), 3);
}

#[test]
fn test_position_tick_no_movement_when_stopped() {
    let mut pos = Position::Stopped { node: 0 };
    let result = pos.tick(5.0);

    assert!(result.is_none());
    assert_eq!(pos, Position::Stopped { node: 0 });
}

#[test]
fn test_position_tick_no_movement_when_zero_distance() {
    let mut pos = Position::Moving {
        from: 0,
        to: 1,
        remaining_distance: 10.0,
    };
    let result = pos.tick(0.0);

    assert!(result.is_none());
    assert_eq!(
        pos,
        Position::Moving {
            from: 0,
            to: 1,
            remaining_distance: 10.0,
        }
    );
}

#[test]
fn test_position_tick_partial_movement() {
    let mut pos = Position::Moving {
        from: 0,
        to: 1,
        remaining_distance: 10.0,
    };
    let result = pos.tick(3.0);

    assert!(result.is_none());
    assert_eq!(
        pos,
        Position::Moving {
            from: 0,
            to: 1,
            remaining_distance: 7.0,
        }
    );
}

#[test]
fn test_position_tick_exact_arrival() {
    let mut pos = Position::Moving {
        from: 0,
        to: 1,
        remaining_distance: 5.0,
    };
    let result = pos.tick(5.0);

    assert!(result.is_none());
    assert_eq!(pos, Position::Stopped { node: 1 });
}

#[test]
fn test_position_tick_overshoot_with_overflow() {
    let mut pos = Position::Moving {
        from: 0,
        to: 1,
        remaining_distance: 3.0,
    };
    let result = pos.tick(8.0);

    assert_eq!(result, Some(5.0));
    assert_eq!(pos, Position::Stopped { node: 1 });
}

#[test]
fn test_position_get_pixel_position_stopped() {
    let graph = common::create_test_graph();
    let pos = Position::Stopped { node: 0 };

    let pixel_pos = pos.get_pixel_position(&graph).unwrap();
    let expected = Vec2::new(
        0.0 + pacman::constants::BOARD_PIXEL_OFFSET.x as f32,
        0.0 + pacman::constants::BOARD_PIXEL_OFFSET.y as f32,
    );

    assert_eq!(pixel_pos, expected);
}

#[test]
fn test_position_get_pixel_position_moving() {
    let graph = common::create_test_graph();
    let pos = Position::Moving {
        from: 0,
        to: 1,
        remaining_distance: 8.0, // Halfway through a 16-unit edge
    };

    let pixel_pos = pos.get_pixel_position(&graph).unwrap();
    // Should be halfway between (0,0) and (16,0), so at (8,0) plus offset
    let expected = Vec2::new(
        8.0 + pacman::constants::BOARD_PIXEL_OFFSET.x as f32,
        0.0 + pacman::constants::BOARD_PIXEL_OFFSET.y as f32,
    );

    assert_eq!(pixel_pos, expected);
}

#[test]
fn test_velocity_basic_properties() {
    let velocity = Velocity {
        speed: 2.5,
        direction: Direction::Up,
    };

    assert_eq!(velocity.speed, 2.5);
    assert_eq!(velocity.direction, Direction::Up);
}

#[test]
fn test_buffered_direction_none() {
    let buffered = BufferedDirection::None;
    assert_eq!(buffered, BufferedDirection::None);
}

#[test]
fn test_buffered_direction_some() {
    let buffered = BufferedDirection::Some {
        direction: Direction::Left,
        remaining_time: 0.5,
    };

    if let BufferedDirection::Some {
        direction,
        remaining_time,
    } = buffered
    {
        assert_eq!(direction, Direction::Left);
        assert_eq!(remaining_time, 0.5);
    } else {
        panic!("Expected BufferedDirection::Some");
    }
}
