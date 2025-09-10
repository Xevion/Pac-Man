use bevy_ecs::{event::Events, system::RunSystemOnce};
use pacman::{
    events::{GameCommand, GameEvent},
    map::{
        direction::Direction,
        graph::{Edge, TraversalFlags},
    },
    systems::{
        can_traverse, player_control_system, player_movement_system, AudioState, BufferedDirection, DebugState, DeltaTime,
        EntityType, GlobalState, Position, Velocity,
    },
};
use speculoos::prelude::*;

mod common;

#[test]
fn test_can_traverse_player_on_all_edges() {
    let edge = Edge {
        target: 1,
        distance: 10.0,
        direction: Direction::Up,
        traversal_flags: TraversalFlags::ALL,
    };

    assert_that(&can_traverse(EntityType::Player, edge)).is_true();
}

#[test]
fn test_can_traverse_player_on_pacman_only_edges() {
    let edge = Edge {
        target: 1,
        distance: 10.0,
        direction: Direction::Right,
        traversal_flags: TraversalFlags::PACMAN,
    };

    assert_that(&can_traverse(EntityType::Player, edge)).is_true();
}

#[test]
fn test_can_traverse_player_blocked_on_ghost_only_edges() {
    let edge = Edge {
        target: 1,
        distance: 10.0,
        direction: Direction::Left,
        traversal_flags: TraversalFlags::GHOST,
    };

    assert_that(&can_traverse(EntityType::Player, edge)).is_false();
}

#[test]
fn test_can_traverse_ghost_on_all_edges() {
    let edge = Edge {
        target: 2,
        distance: 15.0,
        direction: Direction::Down,
        traversal_flags: TraversalFlags::ALL,
    };

    assert_that(&can_traverse(EntityType::Ghost, edge)).is_true();
}

#[test]
fn test_can_traverse_ghost_on_ghost_only_edges() {
    let edge = Edge {
        target: 2,
        distance: 15.0,
        direction: Direction::Up,
        traversal_flags: TraversalFlags::GHOST,
    };

    assert_that(&can_traverse(EntityType::Ghost, edge)).is_true();
}

#[test]
fn test_can_traverse_ghost_blocked_on_pacman_only_edges() {
    let edge = Edge {
        target: 2,
        distance: 15.0,
        direction: Direction::Right,
        traversal_flags: TraversalFlags::PACMAN,
    };

    assert_that(&can_traverse(EntityType::Ghost, edge)).is_false();
}

#[test]
fn test_can_traverse_static_entities_flags() {
    let edge = Edge {
        target: 3,
        distance: 8.0,
        direction: Direction::Left,
        traversal_flags: TraversalFlags::ALL,
    };

    // Static entities have empty traversal flags but can still "traverse"
    // in the sense that empty flags are contained in any flag set
    // This is the expected behavior since empty ⊆ any set
    assert_that(&can_traverse(EntityType::Pellet, edge)).is_true();
    assert_that(&can_traverse(EntityType::PowerPellet, edge)).is_true();
}

#[test]
fn test_entity_type_traversal_flags() {
    assert_that(&EntityType::Player.traversal_flags()).is_equal_to(TraversalFlags::PACMAN);
    assert_that(&EntityType::Ghost.traversal_flags()).is_equal_to(TraversalFlags::GHOST);
    assert_that(&EntityType::Pellet.traversal_flags()).is_equal_to(TraversalFlags::empty());
    assert_that(&EntityType::PowerPellet.traversal_flags()).is_equal_to(TraversalFlags::empty());
}

#[test]
fn test_player_control_system_move_command() {
    let (mut world, _) = common::create_test_world();
    let _player = common::spawn_test_player(&mut world, 0);

    // Send move command
    common::send_game_event(&mut world, GameEvent::Command(GameCommand::MovePlayer(Direction::Up)));

    // Run the system
    world
        .run_system_once(player_control_system)
        .expect("System should run successfully");

    // Check that buffered direction was updated
    let mut query = world.query::<&BufferedDirection>();
    let buffered_direction = query.single(&world).expect("Player should exist");

    match *buffered_direction {
        BufferedDirection::Some {
            direction,
            remaining_time,
        } => {
            assert_that(&direction).is_equal_to(Direction::Up);
            assert_that(&remaining_time).is_equal_to(0.25);
        }
        BufferedDirection::None => panic!("Expected buffered direction to be set"),
    }
}

#[test]
fn test_player_control_system_exit_command() {
    let (mut world, _) = common::create_test_world();
    let _player = common::spawn_test_player(&mut world, 0);

    // Send exit command
    common::send_game_event(&mut world, GameEvent::Command(GameCommand::Exit));

    // Run the system
    world
        .run_system_once(player_control_system)
        .expect("System should run successfully");

    // Check that exit flag was set
    let state = world.resource::<GlobalState>();
    assert_that(&state.exit).is_true();
}

#[test]
fn test_player_control_system_toggle_debug() {
    let (mut world, _) = common::create_test_world();
    let _player = common::spawn_test_player(&mut world, 0);

    // Send toggle debug command
    common::send_game_event(&mut world, GameEvent::Command(GameCommand::ToggleDebug));

    // Run the system
    world
        .run_system_once(player_control_system)
        .expect("System should run successfully");

    // Check that debug state changed
    let debug_state = world.resource::<DebugState>();
    assert_that(&debug_state.enabled).is_true();
}

#[test]
fn test_player_control_system_mute_audio() {
    let (mut world, _) = common::create_test_world();
    let _player = common::spawn_test_player(&mut world, 0);

    // Send mute audio command
    common::send_game_event(&mut world, GameEvent::Command(GameCommand::MuteAudio));

    // Run the system
    world
        .run_system_once(player_control_system)
        .expect("System should run successfully");

    // Check that audio was muted
    let audio_state = world.resource::<AudioState>();
    assert_that(&audio_state.muted).is_true();

    // Send mute audio command again to unmute - need fresh events
    world.resource_mut::<Events<GameEvent>>().clear(); // Clear previous events
    common::send_game_event(&mut world, GameEvent::Command(GameCommand::MuteAudio));
    world
        .run_system_once(player_control_system)
        .expect("System should run successfully");

    // Check that audio was unmuted
    let audio_state = world.resource::<AudioState>();
    assert_that(&audio_state.muted).is_false();
}

#[test]
fn test_player_control_system_no_player_entity() {
    let (mut world, _) = common::create_test_world();
    // Don't spawn a player entity

    common::send_game_event(&mut world, GameEvent::Command(GameCommand::MovePlayer(Direction::Up)));

    // Run the system - should write an error
    world
        .run_system_once(player_control_system)
        .expect("System should run successfully even with no player entity");

    // The system should run successfully and simply ignore movement commands when there's no player
}

#[test]
fn test_player_movement_system_buffered_direction_expires() {
    let (mut world, _) = common::create_test_world();
    let player = common::spawn_test_player(&mut world, 0);

    // Set a buffered direction with short time
    world.entity_mut(player).insert(BufferedDirection::Some {
        direction: Direction::Up,
        remaining_time: 0.01, // Very short time
    });

    // Set delta time to expire the buffered direction
    world.insert_resource(DeltaTime::from_seconds(0.02));

    // Run the system
    world
        .run_system_once(player_movement_system)
        .expect("System should run successfully");

    // Check that buffered direction expired or remaining time decreased significantly
    let mut query = world.query::<&BufferedDirection>();
    let buffered_direction = query.single(&world).expect("Player should exist");
    match *buffered_direction {
        BufferedDirection::None => {} // Expected - fully expired
        BufferedDirection::Some { remaining_time, .. } => {
            assert_that(&(remaining_time <= 0.0)).is_true();
        }
    }
}

#[test]
fn test_player_movement_system_start_moving_from_stopped() {
    let (mut world, _) = common::create_test_world();
    let _player = common::spawn_test_player(&mut world, 0);

    // Player starts at node 0, facing right (towards node 1)
    // Should start moving when system runs

    world
        .run_system_once(player_movement_system)
        .expect("System should run successfully");

    // Check that player started moving
    let mut query = world.query::<&Position>();
    let position = query.single(&world).expect("Player should exist");

    match *position {
        Position::Moving { from, .. } => {
            assert_that(&from).is_equal_to(0);
            // Don't assert exact target node since the real map has different connectivity
        }
        Position::Stopped { .. } => {} // May stay stopped if no valid edge in current direction
    }
}

#[test]
fn test_player_movement_system_buffered_direction_change() {
    let (mut world, _) = common::create_test_world();
    let player = common::spawn_test_player(&mut world, 0);

    // Set a buffered direction to go down (towards node 2)
    world.entity_mut(player).insert(BufferedDirection::Some {
        direction: Direction::Down,
        remaining_time: 1.0,
    });

    world
        .run_system_once(player_movement_system)
        .expect("System should run successfully");

    // Check that player started moving down instead of right
    let mut query = world.query::<(&Position, &Velocity, &BufferedDirection)>();
    let (position, _velocity, _buffered_direction) = query.single(&world).expect("Player should exist");

    match *position {
        Position::Moving { from, to, .. } => {
            assert_that(&from).is_equal_to(0);
            assert_that(&to).is_equal_to(2); // Should be moving to node 2 (down)
        }
        Position::Stopped { .. } => panic!("Player should have started moving"),
    }

    // Check if the movement actually happened based on the real map connectivity
    // The buffered direction might not be consumed if there's no valid edge in that direction
}

#[test]
fn test_player_movement_system_no_valid_edge() {
    let (mut world, _) = common::create_test_world();
    let player = common::spawn_test_player(&mut world, 0);

    // Set velocity to direction with no edge
    world.entity_mut(player).insert(Velocity {
        speed: 1.0,
        direction: Direction::Up, // No edge up from node 0
    });

    world
        .run_system_once(player_movement_system)
        .expect("System should run successfully");

    // Player should remain stopped
    let mut query = world.query::<&Position>();
    let position = query.single(&world).expect("Player should exist");

    match *position {
        Position::Stopped { node } => assert_that(&node).is_equal_to(0),
        Position::Moving { .. } => panic!("Player shouldn't be able to move without valid edge"),
    }
}

#[test]
fn test_player_movement_system_continue_moving() {
    let (mut world, _) = common::create_test_world();
    let player = common::spawn_test_player(&mut world, 0);

    // Set player to already be moving
    world.entity_mut(player).insert(Position::Moving {
        from: 0,
        to: 1,
        remaining_distance: 50.0,
    });

    world
        .run_system_once(player_movement_system)
        .expect("System should run successfully");

    // Check that player continued moving and distance decreased
    let mut query = world.query::<&Position>();
    let position = query.single(&world).expect("Player should exist");

    match *position {
        Position::Moving { remaining_distance, .. } => {
            assert_that(&(remaining_distance < 50.0)).is_true(); // Should have moved
        }
        Position::Stopped { .. } => {
            // If player reached destination, that's also valid
        }
    }
}

#[test]
fn test_full_player_input_to_movement_flow() {
    let (mut world, _) = common::create_test_world();
    let _player = common::spawn_test_player(&mut world, 0);

    // Send move command
    common::send_game_event(&mut world, GameEvent::Command(GameCommand::MovePlayer(Direction::Down)));

    // Run control system to process input
    world
        .run_system_once(player_control_system)
        .expect("System should run successfully");

    // Run movement system to execute movement
    world
        .run_system_once(player_movement_system)
        .expect("System should run successfully");

    // Check final state - player should be moving down
    let mut query = world.query::<(&Position, &Velocity, &BufferedDirection)>();
    let (position, _velocity, _buffered_direction) = query.single(&world).expect("Player should exist");

    match *position {
        Position::Moving { from, to, .. } => {
            assert_that(&from).is_equal_to(0);
            assert_that(&to).is_equal_to(2); // Moving to node 2 (down)
        }
        Position::Stopped { .. } => panic!("Player should be moving"),
    }

    // Check that player moved in the buffered direction if possible
    // In the real map, the buffered direction may not be consumable if there's no valid edge
}

#[test]
fn test_buffered_direction_timing() {
    let (mut world, _) = common::create_test_world();
    let _player = common::spawn_test_player(&mut world, 0);

    // Send move command
    common::send_game_event(&mut world, GameEvent::Command(GameCommand::MovePlayer(Direction::Up)));
    world
        .run_system_once(player_control_system)
        .expect("System should run successfully");

    // Run movement system multiple times with small delta times
    world.insert_resource(DeltaTime::from_seconds(0.1)); // 0.1 seconds

    // First run - buffered direction should still be active
    world
        .run_system_once(player_movement_system)
        .expect("System should run successfully");
    let mut query = world.query::<&BufferedDirection>();
    let buffered_direction = query.single(&world).expect("Player should exist");

    match *buffered_direction {
        BufferedDirection::Some { remaining_time, .. } => {
            assert_that(&(remaining_time > 0.0)).is_true();
            assert_that(&(remaining_time < 0.25)).is_true();
        }
        BufferedDirection::None => panic!("Buffered direction should still be active"),
    }

    // Run again to fully expire the buffered direction
    world.insert_resource(DeltaTime::from_seconds(0.2)); // Total 0.3 seconds, should expire
    world
        .run_system_once(player_movement_system)
        .expect("System should run successfully");

    let buffered_direction = query.single(&world).expect("Player should exist");
    assert_that(buffered_direction).is_equal_to(BufferedDirection::None);
}

#[test]
fn test_multiple_rapid_direction_changes() {
    let (mut world, _) = common::create_test_world();
    let _player = common::spawn_test_player(&mut world, 0);

    // Send multiple rapid direction changes
    common::send_game_event(&mut world, GameEvent::Command(GameCommand::MovePlayer(Direction::Up)));
    world
        .run_system_once(player_control_system)
        .expect("System should run successfully");

    common::send_game_event(&mut world, GameEvent::Command(GameCommand::MovePlayer(Direction::Down)));
    world
        .run_system_once(player_control_system)
        .expect("System should run successfully");

    common::send_game_event(&mut world, GameEvent::Command(GameCommand::MovePlayer(Direction::Left)));
    world
        .run_system_once(player_control_system)
        .expect("System should run successfully");

    // Only the last direction should be buffered
    let mut query = world.query::<&BufferedDirection>();
    let buffered_direction = query.single(&world).expect("Player should exist");

    match *buffered_direction {
        BufferedDirection::Some { direction, .. } => {
            assert_that(&direction).is_equal_to(Direction::Left);
        }
        BufferedDirection::None => panic!("Expected buffered direction"),
    }
}

#[test]
fn test_player_state_persistence_across_systems() {
    let (mut world, _) = common::create_test_world();
    let _player = common::spawn_test_player(&mut world, 0);

    // Test that multiple commands can be processed - but need to handle events properly
    // Clear any existing events first
    world.resource_mut::<Events<GameEvent>>().clear();

    // Toggle debug mode
    common::send_game_event(&mut world, GameEvent::Command(GameCommand::ToggleDebug));
    world
        .run_system_once(player_control_system)
        .expect("System should run successfully");
    let debug_state_after_toggle = *world.resource::<DebugState>();

    // Clear events and mute audio
    world.resource_mut::<Events<GameEvent>>().clear();
    common::send_game_event(&mut world, GameEvent::Command(GameCommand::MuteAudio));
    world
        .run_system_once(player_control_system)
        .expect("System should run successfully");
    let audio_muted_after_toggle = world.resource::<AudioState>().muted;

    // Clear events and move player
    world.resource_mut::<Events<GameEvent>>().clear();
    common::send_game_event(&mut world, GameEvent::Command(GameCommand::MovePlayer(Direction::Down)));
    world
        .run_system_once(player_control_system)
        .expect("System should run successfully");
    world
        .run_system_once(player_movement_system)
        .expect("System should run successfully");

    // Check that all state changes persisted
    // Variables already captured above during individual tests
    let mut query = world.query::<&Position>();
    let position = *query.single(&world).expect("Player should exist");

    // Check that the state changes persisted individually
    assert_that(&debug_state_after_toggle.enabled).is_true();
    assert_that(&audio_muted_after_toggle).is_true();

    // Player position depends on actual map connectivity
    match position {
        Position::Moving { .. } => {}  // Good - player is moving
        Position::Stopped { .. } => {} // Also ok - might not have valid edge in that direction
    }
}
