use assert2::check;
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

    check!(stopped_pos.is_at_node());
    check!(!moving_pos.is_at_node());
}

#[test]
fn test_position_current_node() {
    let stopped_pos = Position::Stopped { node: 5 };
    let moving_pos = Position::Moving {
        from: 3,
        to: 7,
        remaining_distance: 12.0,
    };

    check!(stopped_pos.current_node() == 5);
    check!(moving_pos.current_node() == 3);
}

#[test]
fn test_position_tick_no_movement_when_stopped() {
    let mut pos = Position::Stopped { node: 0 };
    let result = pos.tick(5.0);

    check!(result.is_none());
    check!(pos == Position::Stopped { node: 0 });
}

#[test]
fn test_position_tick_no_movement_when_zero_distance() {
    let mut pos = Position::Moving {
        from: 0,
        to: 1,
        remaining_distance: 10.0,
    };
    let result = pos.tick(0.0);

    check!(result.is_none());
    check!(
        pos == Position::Moving {
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

    check!(result.is_none());
    check!(
        pos == Position::Moving {
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

    check!(result.is_none());
    check!(pos == Position::Stopped { node: 1 });
}

#[test]
fn test_position_tick_overshoot_with_overflow() {
    let mut pos = Position::Moving {
        from: 0,
        to: 1,
        remaining_distance: 3.0,
    };
    let result = pos.tick(8.0);

    check!(result == Some(5.0));
    check!(pos == Position::Stopped { node: 1 });
}

#[test]
fn test_position_get_pixel_position_stopped() {
    let graph = common::create_test_graph();
    let pos = Position::Stopped { node: 0 };

    let pixel_pos = pos.get_pixel_position(&graph).unwrap();
    // Maze-local: node 0 sits at the playfield origin, no HUD offset baked in.
    let expected = Vec2::new(0.0, 0.0);

    check!(pixel_pos == expected);
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
    // Maze-local: halfway between (0,0) and (16,0).
    let expected = Vec2::new(8.0, 0.0);

    check!(pixel_pos == expected);
}

#[test]
fn test_velocity_basic_properties() {
    let velocity = Velocity {
        speed: 2.5,
        direction: Direction::Up,
    };

    check!(velocity.speed == 2.5);
    check!(velocity.direction == Direction::Up);
}

#[test]
fn test_buffered_direction_none() {
    let buffered = BufferedDirection::None;
    check!(buffered == BufferedDirection::None);
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
        check!(direction == Direction::Left);
        check!(remaining_time == 0.5);
    } else {
        panic!("Expected BufferedDirection::Some");
    }
}
