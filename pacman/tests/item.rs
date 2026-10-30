use assert2::check;
use bevy_ecs::entity::Entity;
use pacman::{
    events::CollisionTrigger,
    systems::{
        common::{EntityType, ScoreResource},
        ghost::GhostState,
        movement::Position,
        state::Session,
    },
};

mod common;

#[test]
fn test_calculate_score_for_item() {
    check!(EntityType::Pellet.score_value() < EntityType::PowerPellet.score_value());
    check!(EntityType::Pellet.score_value().is_some());
    check!(EntityType::PowerPellet.score_value().is_some());
    check!(EntityType::Player.score_value().is_none());
    check!(EntityType::Ghost.score_value().is_none());
}

#[test]
fn test_is_collectible_item() {
    // Collectible
    check!(EntityType::Pellet.is_collectible());
    check!(EntityType::PowerPellet.is_collectible());

    // Non-collectible
    check!(!EntityType::Player.is_collectible());
    check!(!EntityType::Ghost.is_collectible());
}

#[test]
fn test_item_system_pellet_collection() {
    let (mut world, mut _schedule) = common::create_test_world();
    let pellet = common::spawn_test_item(&mut world, 1, EntityType::Pellet);

    // Send collision event
    common::trigger_collision(&mut world, CollisionTrigger::ItemCollision { item: pellet });

    world.flush();

    // Check that score was updated
    let score = &world.resource::<Session>().score;
    check!(score.value() == 10);

    // Check that the pellet was despawned (query should return empty)
    let item_count = world
        .query::<&EntityType>()
        .iter(&world)
        .filter(|&entity_type| matches!(entity_type, EntityType::Pellet))
        .count();
    check!(item_count == 0);
}

#[test]
fn test_item_system_power_pellet_collection() {
    let (mut world, mut _schedule) = common::create_test_world();
    let power_pellet = common::spawn_test_item(&mut world, 1, EntityType::PowerPellet);

    common::trigger_collision(&mut world, CollisionTrigger::ItemCollision { item: power_pellet });

    world.flush();

    // Check that score was updated with power pellet value
    let score = &world.resource::<Session>().score;
    check!(score.value() == 50);

    // Check that the power pellet was despawned (query should return empty)
    let item_count = world
        .query::<&EntityType>()
        .iter(&world)
        .filter(|&entity_type| matches!(entity_type, EntityType::PowerPellet))
        .count();
    check!(item_count == 0);
}

#[test]
fn test_item_system_multiple_collections() {
    let (mut world, mut _schedule) = common::create_test_world();
    let pellet1 = common::spawn_test_item(&mut world, 1, EntityType::Pellet);
    let pellet2 = common::spawn_test_item(&mut world, 2, EntityType::Pellet);
    let power_pellet = common::spawn_test_item(&mut world, 3, EntityType::PowerPellet);

    // Send multiple collision events
    common::trigger_collision(&mut world, CollisionTrigger::ItemCollision { item: pellet1 });
    common::trigger_collision(&mut world, CollisionTrigger::ItemCollision { item: pellet2 });
    common::trigger_collision(&mut world, CollisionTrigger::ItemCollision { item: power_pellet });

    world.flush();

    // Check final score: 2 pellets (20) + 1 power pellet (50) = 70
    let score = &world.resource::<Session>().score;
    check!(score.value() == 70);

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
    check!(pellet_count == 0);
    check!(power_pellet_count == 0);
}

#[test]
fn test_item_system_ignores_non_item_collisions() {
    let (mut world, mut _schedule) = common::create_test_world();

    // Create a ghost entity (not an item)
    let ghost = world.spawn((Position::Stopped { node: 2 }, EntityType::Ghost)).id();

    // Initial score
    let initial_score = world.resource::<Session>().score.value();

    // Send collision event between pacman and ghost
    common::trigger_collision(&mut world, CollisionTrigger::ItemCollision { item: ghost });

    world.flush();

    // Score should remain unchanged
    let score = &world.resource::<Session>().score;
    check!(score.value() == initial_score);

    // Ghost should still exist (not despawned)
    let ghost_count = world
        .query::<&EntityType>()
        .iter(&world)
        .filter(|&entity_type| matches!(entity_type, EntityType::Ghost))
        .count();
    check!(ghost_count == 1);
}

#[test]
fn test_item_system_no_collision_events() {
    let (mut world, mut _schedule) = common::create_test_world();
    let _pacman = common::spawn_test_pacman(&mut world, 0);
    let _pellet = common::spawn_test_item(&mut world, 1, EntityType::Pellet);

    let initial_score = world.resource::<Session>().score.value();

    // Run system without any collision events
    world.flush();

    // Nothing should change
    let score = &world.resource::<Session>().score;
    check!(score.value() == initial_score);
    let pellet_count = world
        .query::<&EntityType>()
        .iter(&world)
        .filter(|&entity_type| matches!(entity_type, EntityType::Pellet))
        .count();
    check!(pellet_count == 1);
}

#[test]
fn test_item_system_collision_with_missing_entity() {
    let (mut world, mut _schedule) = common::create_test_world();

    // Create a fake entity ID that doesn't exist
    let fake_entity = Entity::from_raw_u32(999).unwrap();

    common::trigger_collision(&mut world, CollisionTrigger::ItemCollision { item: fake_entity });

    // System should handle gracefully and not crash
    world.flush();
    // Score should remain unchanged
    let score = &world.resource::<Session>().score;
    check!(score.value() == 0);
}

#[test]
fn test_item_system_preserves_existing_score() {
    let (mut world, mut _schedule) = common::create_test_world();

    // Set initial score
    world.resource_mut::<Session>().score = ScoreResource::new(100);

    let pellet = common::spawn_test_item(&mut world, 1, EntityType::Pellet);

    common::trigger_collision(&mut world, CollisionTrigger::ItemCollision { item: pellet });

    world.flush();

    // Score should be initial + pellet value
    let score = &world.resource::<Session>().score;
    check!(score.value() == 110);
}

#[test]
fn test_power_pellet_does_not_affect_ghosts_in_eyes_state() {
    let (mut world, mut _schedule) = common::create_test_world();
    let power_pellet = common::spawn_test_item(&mut world, 1, EntityType::PowerPellet);

    // Spawn a ghost in Eyes state (returning to ghost house)
    let eyes_ghost = common::spawn_test_ghost(&mut world, 2, GhostState::Eyes);

    // Spawn a ghost in Normal state
    let normal_ghost = common::spawn_test_ghost(&mut world, 3, GhostState::Active { frightened: None });

    common::trigger_collision(&mut world, CollisionTrigger::ItemCollision { item: power_pellet });

    world.flush();

    // Check that the power pellet was collected and score updated
    let score = &world.resource::<Session>().score;
    check!(score.value() == 50);

    // Check that the power pellet was despawned
    let power_pellet_count = world
        .query::<&EntityType>()
        .iter(&world)
        .filter(|&entity_type| matches!(entity_type, EntityType::PowerPellet))
        .count();
    check!(power_pellet_count == 0);

    // Check that the Eyes ghost state was not changed
    let eyes_ghost_state = world.entity(eyes_ghost).get::<GhostState>().unwrap();
    check!(matches!(*eyes_ghost_state, GhostState::Eyes));

    // Check that the Normal ghost state was changed to Frightened
    let normal_ghost_state = world.entity(normal_ghost).get::<GhostState>().unwrap();
    check!(matches!(*normal_ghost_state, GhostState::Active { frightened: Some(_) }));
}
