use bevy_ecs::{event::Events, prelude::*, system::RunSystemOnce, world::World};

use pacman::{
    events::GameEvent,
    map::builder::Map,
    systems::{
        audio::AudioEvent,
        components::{AudioState, EntityType, ItemCollider, PacmanCollider, ScoreResource},
        item::{is_valid_item_collision, item_system},
        movement::Position,
    },
};

#[test]
fn test_calculate_score_for_item() {
    assert!(EntityType::Pellet.score_value() < EntityType::PowerPellet.score_value());
    assert!(EntityType::Pellet.score_value().is_some());
    assert!(EntityType::PowerPellet.score_value().is_some());
    assert!(EntityType::Pellet.score_value().unwrap() < EntityType::PowerPellet.score_value().unwrap());
    assert!(EntityType::Player.score_value().is_none());
    assert!(EntityType::Ghost.score_value().is_none());
}

#[test]
fn test_is_collectible_item() {
    // Collectible
    assert!(EntityType::Pellet.is_collectible());
    assert!(EntityType::PowerPellet.is_collectible());

    // Non-collectible
    assert!(!EntityType::Player.is_collectible());
    assert!(!EntityType::Ghost.is_collectible());
}

#[test]
fn test_is_valid_item_collision() {
    // Player-item collisions should be valid
    assert!(is_valid_item_collision(EntityType::Player, EntityType::Pellet));
    assert!(is_valid_item_collision(EntityType::Player, EntityType::PowerPellet));
    assert!(is_valid_item_collision(EntityType::Pellet, EntityType::Player));
    assert!(is_valid_item_collision(EntityType::PowerPellet, EntityType::Player));

    // Non-player-item collisions should be invalid
    assert!(!is_valid_item_collision(EntityType::Player, EntityType::Ghost));
    assert!(!is_valid_item_collision(EntityType::Ghost, EntityType::Pellet));
    assert!(!is_valid_item_collision(EntityType::Pellet, EntityType::PowerPellet));
    assert!(!is_valid_item_collision(EntityType::Player, EntityType::Player));
}

fn create_test_world() -> World {
    let mut world = World::new();

    // Add required resources
    world.insert_resource(ScoreResource(0));
    world.insert_resource(AudioState::default());
    world.insert_resource(Events::<GameEvent>::default());
    world.insert_resource(Events::<AudioEvent>::default());
    world.insert_resource(Events::<pacman::error::GameError>::default());

    // Add a minimal test map
    world.insert_resource(create_test_map());

    world
}

fn create_test_map() -> Map {
    use pacman::constants::RAW_BOARD;
    Map::new(RAW_BOARD).expect("Failed to create test map")
}

fn spawn_test_pacman(world: &mut World) -> Entity {
    world
        .spawn((Position::Stopped { node: 0 }, EntityType::Player, PacmanCollider))
        .id()
}

fn spawn_test_item(world: &mut World, item_type: EntityType) -> Entity {
    world.spawn((Position::Stopped { node: 1 }, item_type, ItemCollider)).id()
}

fn send_collision_event(world: &mut World, entity1: Entity, entity2: Entity) {
    let mut events = world.resource_mut::<Events<GameEvent>>();
    events.send(GameEvent::Collision(entity1, entity2));
}

#[test]
fn test_item_system_pellet_collection() {
    let mut world = create_test_world();
    let pacman = spawn_test_pacman(&mut world);
    let pellet = spawn_test_item(&mut world, EntityType::Pellet);

    // Send collision event
    send_collision_event(&mut world, pacman, pellet);

    // Run the item system
    world.run_system_once(item_system).expect("System should run successfully");

    // Check that score was updated
    let score = world.resource::<ScoreResource>();
    assert_eq!(score.0, 10);

    // Check that the pellet was despawned (query should return empty)
    let item_count = world
        .query::<&EntityType>()
        .iter(&world)
        .filter(|&entity_type| matches!(entity_type, EntityType::Pellet))
        .count();
    assert_eq!(item_count, 0);
}

#[test]
fn test_item_system_power_pellet_collection() {
    let mut world = create_test_world();
    let pacman = spawn_test_pacman(&mut world);
    let power_pellet = spawn_test_item(&mut world, EntityType::PowerPellet);

    send_collision_event(&mut world, pacman, power_pellet);

    world.run_system_once(item_system).expect("System should run successfully");

    // Check that score was updated with power pellet value
    let score = world.resource::<ScoreResource>();
    assert_eq!(score.0, 50);

    // Check that the power pellet was despawned (query should return empty)
    let item_count = world
        .query::<&EntityType>()
        .iter(&world)
        .filter(|&entity_type| matches!(entity_type, EntityType::PowerPellet))
        .count();
    assert_eq!(item_count, 0);
}

#[test]
fn test_item_system_multiple_collections() {
    let mut world = create_test_world();
    let pacman = spawn_test_pacman(&mut world);
    let pellet1 = spawn_test_item(&mut world, EntityType::Pellet);
    let pellet2 = spawn_test_item(&mut world, EntityType::Pellet);
    let power_pellet = spawn_test_item(&mut world, EntityType::PowerPellet);

    // Send multiple collision events
    send_collision_event(&mut world, pacman, pellet1);
    send_collision_event(&mut world, pacman, pellet2);
    send_collision_event(&mut world, pacman, power_pellet);

    world.run_system_once(item_system).expect("System should run successfully");

    // Check final score: 2 pellets (20) + 1 power pellet (50) = 70
    let score = world.resource::<ScoreResource>();
    assert_eq!(score.0, 70);

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
    assert_eq!(pellet_count, 0);
    assert_eq!(power_pellet_count, 0);
}

#[test]
fn test_item_system_ignores_non_item_collisions() {
    let mut world = create_test_world();
    let pacman = spawn_test_pacman(&mut world);

    // Create a ghost entity (not an item)
    let ghost = world.spawn((Position::Stopped { node: 2 }, EntityType::Ghost)).id();

    // Initial score
    let initial_score = world.resource::<ScoreResource>().0;

    // Send collision event between pacman and ghost
    send_collision_event(&mut world, pacman, ghost);

    world.run_system_once(item_system).expect("System should run successfully");

    // Score should remain unchanged
    let score = world.resource::<ScoreResource>();
    assert_eq!(score.0, initial_score);

    // Ghost should still exist (not despawned)
    let ghost_count = world
        .query::<&EntityType>()
        .iter(&world)
        .filter(|&entity_type| matches!(entity_type, EntityType::Ghost))
        .count();
    assert_eq!(ghost_count, 1);
}

#[test]
fn test_item_system_wrong_collision_order() {
    let mut world = create_test_world();
    let pacman = spawn_test_pacman(&mut world);
    let pellet = spawn_test_item(&mut world, EntityType::Pellet);

    // Send collision event with entities in reverse order
    send_collision_event(&mut world, pellet, pacman);

    world.run_system_once(item_system).expect("System should run successfully");

    // Should still work correctly
    let score = world.resource::<ScoreResource>();
    assert_eq!(score.0, 10);
    let pellet_count = world
        .query::<&EntityType>()
        .iter(&world)
        .filter(|&entity_type| matches!(entity_type, EntityType::Pellet))
        .count();
    assert_eq!(pellet_count, 0);
}

#[test]
fn test_item_system_no_collision_events() {
    let mut world = create_test_world();
    let _pacman = spawn_test_pacman(&mut world);
    let _pellet = spawn_test_item(&mut world, EntityType::Pellet);

    let initial_score = world.resource::<ScoreResource>().0;

    // Run system without any collision events
    world.run_system_once(item_system).expect("System should run successfully");

    // Nothing should change
    let score = world.resource::<ScoreResource>();
    assert_eq!(score.0, initial_score);
    let pellet_count = world
        .query::<&EntityType>()
        .iter(&world)
        .filter(|&entity_type| matches!(entity_type, EntityType::Pellet))
        .count();
    assert_eq!(pellet_count, 1);
}

#[test]
fn test_item_system_collision_with_missing_entity() {
    let mut world = create_test_world();
    let pacman = spawn_test_pacman(&mut world);

    // Create a fake entity ID that doesn't exist
    let fake_entity = Entity::from_raw(999);

    send_collision_event(&mut world, pacman, fake_entity);

    // System should handle gracefully and not crash
    world
        .run_system_once(item_system)
        .expect("System should handle missing entities gracefully");

    // Score should remain unchanged
    let score = world.resource::<ScoreResource>();
    assert_eq!(score.0, 0);
}

#[test]
fn test_item_system_preserves_existing_score() {
    let mut world = create_test_world();

    // Set initial score
    world.insert_resource(ScoreResource(100));

    let pacman = spawn_test_pacman(&mut world);
    let pellet = spawn_test_item(&mut world, EntityType::Pellet);

    send_collision_event(&mut world, pacman, pellet);

    world.run_system_once(item_system).expect("System should run successfully");

    // Score should be initial + pellet value
    let score = world.resource::<ScoreResource>();
    assert_eq!(score.0, 110);
}
