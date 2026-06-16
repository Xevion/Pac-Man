use bevy_ecs::system::RunSystemOnce;
use bevy_ecs::world::World;
use pacman::events::StageTransition;
use pacman::systems::collision::{check_collision, collision_system, Collider};
use pacman::systems::common::{EntityType, Frozen};
use pacman::systems::ghost::{GhostState, GhostType};
use pacman::systems::movement::Position;
use pacman::systems::state::{enter_ghost_eaten_pause, GameStage, Session};
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
    let (mut world, mut schedule) = common::create_test_world();
    let _pacman = common::spawn_test_pacman(&mut world, 0);
    let _item = common::spawn_test_item(&mut world, 0, EntityType::Pellet);

    // Run collision system - should not panic
    schedule.run(&mut world);
}

#[test]
fn test_collision_system_pacman_ghost() {
    let (mut world, _) = common::create_test_world();
    let _pacman = common::spawn_test_pacman(&mut world, 0);
    let _ghost = common::spawn_test_ghost(&mut world, 0, GhostState::Active { frightened: None });

    // Run collision system - should not panic
    world
        .run_system_once(collision_system)
        .expect("System should run successfully");
}

#[test]
fn test_collision_system_no_collision() {
    let (mut world, mut schedule) = common::create_test_world();
    let _pacman = common::spawn_test_pacman(&mut world, 0);
    let _ghost = common::spawn_test_ghost(&mut world, 1, GhostState::Active { frightened: None }); // Different node

    // Run collision system - should not panic
    schedule.run(&mut world);
}

#[test]
fn test_collision_system_multiple_entities() {
    let (mut world, _) = common::create_test_world();
    let _pacman = common::spawn_test_pacman(&mut world, 0);
    let _item = common::spawn_test_item(&mut world, 0, EntityType::Pellet);
    let _ghost = common::spawn_test_ghost(&mut world, 0, GhostState::Active { frightened: None });

    // Run collision system - should not panic
    world
        .run_system_once(collision_system)
        .expect("System should run successfully");
}

/// The ghost-eaten observer (which replaced the buffered `StageTransition` event) must
/// enter the pause stage and freeze the player and every ghost when triggered.
#[test]
fn ghost_eaten_observer_enters_pause_and_freezes() {
    let mut world = World::new();
    world.insert_resource(Session::default());
    world.resource_mut::<Session>().stage = GameStage::Playing;
    world.add_observer(enter_ghost_eaten_pause);

    let player = common::spawn_test_player(&mut world, 0);
    // The just-eaten ghost is already Eyes; a second, active ghost must still be frozen.
    let eaten = common::spawn_test_ghost(&mut world, 0, GhostState::Eyes);
    let bystander = common::spawn_test_ghost(&mut world, 1, GhostState::Active { frightened: None });

    world.trigger(StageTransition::GhostEatenPause {
        ghost_entity: eaten,
        ghost_type: GhostType::Blinky,
    });
    world.flush();

    assert_that(&matches!(
        world.resource::<Session>().stage,
        GameStage::GhostEatenPause { .. }
    ))
    .is_true();
    assert_that(&world.get::<Frozen>(player).is_some()).is_true();
    assert_that(&world.get::<Frozen>(eaten).is_some()).is_true();
    assert_that(&world.get::<Frozen>(bystander).is_some()).is_true();
}
