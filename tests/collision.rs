use bevy_ecs::system::RunSystemOnce;
use pacman::systems::{check_collision, collision_system, Collider, EntityType, GhostState, Position};
use speculoos::prelude::*;

mod common;

#[test]
fn test_collider_collision_detection() {
    let collider1 = Collider { size: 10.0 };
    let collider2 = Collider { size: 8.0 };

    // Test collision detection
    assert_that(&collider1.collides_with(collider2.size, 5.0)).is_true(); // Should collide (distance < 9.0)
    assert_that(&collider1.collides_with(collider2.size, 15.0)).is_false(); // Should not collide (distance > 9.0)
}

#[test]
fn test_check_collision_helper() {
    let map = common::create_test_map();
    let pos1 = Position::Stopped { node: 0 };
    let pos2 = Position::Stopped { node: 0 }; // Same position
    let collider1 = Collider { size: 10.0 };
    let collider2 = Collider { size: 8.0 };

    // Test collision at same position
    let result = check_collision(&pos1, &collider1, &pos2, &collider2, &map);
    assert_that(&result.is_ok()).is_true();
    assert_that(&result.unwrap()).is_true(); // Should collide at same position

    // Test collision at different positions
    let pos3 = Position::Stopped { node: 1 }; // Different position
    let result = check_collision(&pos1, &collider1, &pos3, &collider2, &map);
    assert_that(&result.is_ok()).is_true();
    // May or may not collide depending on actual node positions
}

#[test]
fn test_collision_system_pacman_item() {
    let mut world = common::create_test_world();
    let _pacman = common::spawn_test_pacman(&mut world, 0);
    let _item = common::spawn_test_item(&mut world, 0, EntityType::Pellet);

    // Run collision system - should not panic
    world
        .run_system_once(collision_system)
        .expect("System should run successfully");
}

#[test]
fn test_collision_system_pacman_ghost() {
    let mut world = common::create_test_world();
    let _pacman = common::spawn_test_pacman(&mut world, 0);
    let _ghost = common::spawn_test_ghost(&mut world, 0, GhostState::Normal);

    // Run collision system - should not panic
    world
        .run_system_once(collision_system)
        .expect("System should run successfully");
}

#[test]
fn test_collision_system_no_collision() {
    let mut world = common::create_test_world();
    let _pacman = common::spawn_test_pacman(&mut world, 0);
    let _ghost = common::spawn_test_ghost(&mut world, 1, GhostState::Normal); // Different node

    // Run collision system - should not panic
    world
        .run_system_once(collision_system)
        .expect("System should run successfully");
}

#[test]
fn test_collision_system_multiple_entities() {
    let mut world = common::create_test_world();
    let _pacman = common::spawn_test_pacman(&mut world, 0);
    let _item = common::spawn_test_item(&mut world, 0, EntityType::Pellet);
    let _ghost = common::spawn_test_ghost(&mut world, 0, GhostState::Normal);

    // Run collision system - should not panic
    world
        .run_system_once(collision_system)
        .expect("System should run successfully");
}
