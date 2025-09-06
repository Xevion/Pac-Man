use bevy_ecs::{entity::Entity, system::RunSystemOnce};
use pacman::systems::{is_valid_item_collision, item_system, EntityType, GhostState, Position, ScoreResource};
use speculoos::prelude::*;

mod common;

#[test]
fn test_calculate_score_for_item() {
    assert_that(&(EntityType::Pellet.score_value() < EntityType::PowerPellet.score_value())).is_true();
    assert_that(&EntityType::Pellet.score_value().is_some()).is_true();
    assert_that(&EntityType::PowerPellet.score_value().is_some()).is_true();
    assert_that(&EntityType::Player.score_value().is_none()).is_true();
    assert_that(&EntityType::Ghost.score_value().is_none()).is_true();
}

#[test]
fn test_is_collectible_item() {
    // Collectible
    assert_that(&EntityType::Pellet.is_collectible()).is_true();
    assert_that(&EntityType::PowerPellet.is_collectible()).is_true();

    // Non-collectible
    assert_that(&EntityType::Player.is_collectible()).is_false();
    assert_that(&EntityType::Ghost.is_collectible()).is_false();
}

#[test]
fn test_is_valid_item_collision() {
    // Player-item collisions should be valid
    assert_that(&is_valid_item_collision(EntityType::Player, EntityType::Pellet)).is_true();
    assert_that(&is_valid_item_collision(EntityType::Player, EntityType::PowerPellet)).is_true();
    assert_that(&is_valid_item_collision(EntityType::Pellet, EntityType::Player)).is_true();
    assert_that(&is_valid_item_collision(EntityType::PowerPellet, EntityType::Player)).is_true();

    // Non-player-item collisions should be invalid
    assert_that(&is_valid_item_collision(EntityType::Player, EntityType::Ghost)).is_false();
    assert_that(&is_valid_item_collision(EntityType::Ghost, EntityType::Pellet)).is_false();
    assert_that(&is_valid_item_collision(EntityType::Pellet, EntityType::PowerPellet)).is_false();
    assert_that(&is_valid_item_collision(EntityType::Player, EntityType::Player)).is_false();
}

#[test]
fn test_item_system_pellet_collection() {
    let mut world = common::create_test_world();
    let pacman = common::spawn_test_pacman(&mut world, 0);
    let pellet = common::spawn_test_item(&mut world, 1, EntityType::Pellet);

    // Send collision event
    common::send_collision_event(&mut world, pacman, pellet);

    // Run the item system
    world.run_system_once(item_system).expect("System should run successfully");

    // Check that score was updated
    let score = world.resource::<ScoreResource>();
    assert_that(&score.0).is_equal_to(10);

    // Check that the pellet was despawned (query should return empty)
    let item_count = world
        .query::<&EntityType>()
        .iter(&world)
        .filter(|&entity_type| matches!(entity_type, EntityType::Pellet))
        .count();
    assert_that(&item_count).is_equal_to(0);
}

#[test]
fn test_item_system_power_pellet_collection() {
    let mut world = common::create_test_world();
    let pacman = common::spawn_test_pacman(&mut world, 0);
    let power_pellet = common::spawn_test_item(&mut world, 1, EntityType::PowerPellet);

    common::send_collision_event(&mut world, pacman, power_pellet);

    world.run_system_once(item_system).expect("System should run successfully");

    // Check that score was updated with power pellet value
    let score = world.resource::<ScoreResource>();
    assert_that(&score.0).is_equal_to(50);

    // Check that the power pellet was despawned (query should return empty)
    let item_count = world
        .query::<&EntityType>()
        .iter(&world)
        .filter(|&entity_type| matches!(entity_type, EntityType::PowerPellet))
        .count();
    assert_that(&item_count).is_equal_to(0);
}

#[test]
fn test_item_system_multiple_collections() {
    let mut world = common::create_test_world();
    let pacman = common::spawn_test_pacman(&mut world, 0);
    let pellet1 = common::spawn_test_item(&mut world, 1, EntityType::Pellet);
    let pellet2 = common::spawn_test_item(&mut world, 2, EntityType::Pellet);
    let power_pellet = common::spawn_test_item(&mut world, 3, EntityType::PowerPellet);

    // Send multiple collision events
    common::send_collision_event(&mut world, pacman, pellet1);
    common::send_collision_event(&mut world, pacman, pellet2);
    common::send_collision_event(&mut world, pacman, power_pellet);

    world.run_system_once(item_system).expect("System should run successfully");

    // Check final score: 2 pellets (20) + 1 power pellet (50) = 70
    let score = world.resource::<ScoreResource>();
    assert_that(&score.0).is_equal_to(70);

    // Check that all items were despawned
    let pellet_count = world
        .query::<&EntityType>()
        .iter(&world)
        .filter(|&entity_type| matches!(entity_type, EntityType::Pellet))
        .count();
    let power_pellet_count = world
        .query::<&EntityType>()
        .iter(&world)
        .filter(|&entity_type| matches!(entity_type, EntityType::PowerPellet))
        .count();
    assert_that(&pellet_count).is_equal_to(0);
    assert_that(&power_pellet_count).is_equal_to(0);
}

#[test]
fn test_item_system_ignores_non_item_collisions() {
    let mut world = common::create_test_world();
    let pacman = common::spawn_test_pacman(&mut world, 0);

    // Create a ghost entity (not an item)
    let ghost = world.spawn((Position::Stopped { node: 2 }, EntityType::Ghost)).id();

    // Initial score
    let initial_score = world.resource::<ScoreResource>().0;

    // Send collision event between pacman and ghost
    common::send_collision_event(&mut world, pacman, ghost);

    world.run_system_once(item_system).expect("System should run successfully");

    // Score should remain unchanged
    let score = world.resource::<ScoreResource>();
    assert_that(&score.0).is_equal_to(initial_score);

    // Ghost should still exist (not despawned)
    let ghost_count = world
        .query::<&EntityType>()
        .iter(&world)
        .filter(|&entity_type| matches!(entity_type, EntityType::Ghost))
        .count();
    assert_that(&ghost_count).is_equal_to(1);
}

#[test]
fn test_item_system_no_collision_events() {
    let mut world = common::create_test_world();
    let _pacman = common::spawn_test_pacman(&mut world, 0);
    let _pellet = common::spawn_test_item(&mut world, 1, EntityType::Pellet);

    let initial_score = world.resource::<ScoreResource>().0;

    // Run system without any collision events
    world.run_system_once(item_system).expect("System should run successfully");

    // Nothing should change
    let score = world.resource::<ScoreResource>();
    assert_that(&score.0).is_equal_to(initial_score);
    let pellet_count = world
        .query::<&EntityType>()
        .iter(&world)
        .filter(|&entity_type| matches!(entity_type, EntityType::Pellet))
        .count();
    assert_that(&pellet_count).is_equal_to(1);
}

#[test]
fn test_item_system_collision_with_missing_entity() {
    let mut world = common::create_test_world();
    let pacman = common::spawn_test_pacman(&mut world, 0);

    // Create a fake entity ID that doesn't exist
    let fake_entity = Entity::from_raw(999);

    common::send_collision_event(&mut world, pacman, fake_entity);

    // System should handle gracefully and not crash
    world
        .run_system_once(item_system)
        .expect("System should handle missing entities gracefully");

    // Score should remain unchanged
    let score = world.resource::<ScoreResource>();
    assert_that(&score.0).is_equal_to(0);
}

#[test]
fn test_item_system_preserves_existing_score() {
    let mut world = common::create_test_world();

    // Set initial score
    world.insert_resource(ScoreResource(100));

    let pacman = common::spawn_test_pacman(&mut world, 0);
    let pellet = common::spawn_test_item(&mut world, 1, EntityType::Pellet);

    common::send_collision_event(&mut world, pacman, pellet);

    world.run_system_once(item_system).expect("System should run successfully");

    // Score should be initial + pellet value
    let score = world.resource::<ScoreResource>();
    assert_that(&score.0).is_equal_to(110);
}

#[test]
fn test_power_pellet_does_not_affect_ghosts_in_eyes_state() {
    let mut world = common::create_test_world();
    let pacman = common::spawn_test_pacman(&mut world, 0);
    let power_pellet = common::spawn_test_item(&mut world, 1, EntityType::PowerPellet);

    // Spawn a ghost in Eyes state (returning to ghost house)
    let eyes_ghost = common::spawn_test_ghost(&mut world, 2, GhostState::Eyes);

    // Spawn a ghost in Normal state
    let normal_ghost = common::spawn_test_ghost(&mut world, 3, GhostState::Normal);

    common::send_collision_event(&mut world, pacman, power_pellet);

    world.run_system_once(item_system).expect("System should run successfully");

    // Check that the power pellet was collected and score updated
    let score = world.resource::<ScoreResource>();
    assert_that(&score.0).is_equal_to(50);

    // Check that the power pellet was despawned
    let power_pellet_count = world
        .query::<&EntityType>()
        .iter(&world)
        .filter(|&entity_type| matches!(entity_type, EntityType::PowerPellet))
        .count();
    assert_that(&power_pellet_count).is_equal_to(0);

    // Check that the Eyes ghost state was not changed
    let eyes_ghost_state = world.entity(eyes_ghost).get::<GhostState>().unwrap();
    assert_that(&matches!(*eyes_ghost_state, GhostState::Eyes)).is_true();

    // Check that the Normal ghost state was changed to Frightened
    let normal_ghost_state = world.entity(normal_ghost).get::<GhostState>().unwrap();
    assert_that(&matches!(*normal_ghost_state, GhostState::Frightened { .. })).is_true();
}
