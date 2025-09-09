use bevy_ecs::{entity::Entity, system::RunSystemOnce, world::World};
use pacman::systems::{blinking_system, Blinking, DeltaTime, Frozen, Hidden, Renderable};
use speculoos::prelude::*;

mod common;

/// Creates a test world with blinking system resources
fn create_blinking_test_world() -> World {
    let mut world = World::new();
    world.insert_resource(DeltaTime::from_ticks(1));
    world
}

/// Spawns a test entity with blinking and renderable components
fn spawn_blinking_entity(world: &mut World, interval_ticks: u32) -> Entity {
    world
        .spawn((
            Blinking::new(interval_ticks),
            Renderable {
                sprite: common::mock_atlas_tile(1),
                layer: 0,
            },
        ))
        .id()
}

/// Spawns a test entity with blinking, renderable, and hidden components
fn spawn_hidden_blinking_entity(world: &mut World, interval_ticks: u32) -> Entity {
    world
        .spawn((
            Blinking::new(interval_ticks),
            Renderable {
                sprite: common::mock_atlas_tile(1),
                layer: 0,
            },
            Hidden,
        ))
        .id()
}

/// Spawns a test entity with blinking, renderable, and frozen components
fn spawn_frozen_blinking_entity(world: &mut World, interval_ticks: u32) -> Entity {
    world
        .spawn((
            Blinking::new(interval_ticks),
            Renderable {
                sprite: common::mock_atlas_tile(1),
                layer: 0,
            },
            Frozen,
        ))
        .id()
}

/// Spawns a test entity with blinking, renderable, hidden, and frozen components
fn spawn_frozen_hidden_blinking_entity(world: &mut World, interval_ticks: u32) -> Entity {
    world
        .spawn((
            Blinking::new(interval_ticks),
            Renderable {
                sprite: common::mock_atlas_tile(1),
                layer: 0,
            },
            Hidden,
            Frozen,
        ))
        .id()
}

/// Runs the blinking system with the given delta time
fn run_blinking_system(world: &mut World, delta_ticks: u32) {
    world.resource_mut::<DeltaTime>().ticks = delta_ticks;
    world.run_system_once(blinking_system).unwrap();
}

/// Checks if an entity has the Hidden component
fn has_hidden_component(world: &World, entity: Entity) -> bool {
    world.entity(entity).contains::<Hidden>()
}

/// Checks if an entity has the Frozen component
fn has_frozen_component(world: &World, entity: Entity) -> bool {
    world.entity(entity).contains::<Frozen>()
}

#[test]
fn test_blinking_component_creation() {
    let blinking = Blinking::new(10);

    assert_that(&blinking.tick_timer).is_equal_to(0);
    assert_that(&blinking.interval_ticks).is_equal_to(10);
}

#[test]
fn test_blinking_system_normal_interval_no_toggle() {
    let mut world = create_blinking_test_world();
    let entity = spawn_blinking_entity(&mut world, 5);

    // Run system with 3 ticks (less than interval)
    run_blinking_system(&mut world, 3);

    // Entity should not be hidden yet
    assert_that(&has_hidden_component(&world, entity)).is_false();

    // Check that timer was updated
    let blinking = world.entity(entity).get::<Blinking>().unwrap();
    assert_that(&blinking.tick_timer).is_equal_to(3);
}

#[test]
fn test_blinking_system_normal_interval_first_toggle() {
    let mut world = create_blinking_test_world();
    let entity = spawn_blinking_entity(&mut world, 5);

    // Run system with 5 ticks (exactly one interval)
    run_blinking_system(&mut world, 5);

    // Entity should now be hidden
    assert_that(&has_hidden_component(&world, entity)).is_true();

    // Check that timer was reset
    let blinking = world.entity(entity).get::<Blinking>().unwrap();
    assert_that(&blinking.tick_timer).is_equal_to(0);
}

#[test]
fn test_blinking_system_normal_interval_second_toggle() {
    let mut world = create_blinking_test_world();
    let entity = spawn_blinking_entity(&mut world, 5);

    // First toggle: 5 ticks
    run_blinking_system(&mut world, 5);
    assert_that(&has_hidden_component(&world, entity)).is_true();

    // Second toggle: another 5 ticks
    run_blinking_system(&mut world, 5);
    assert_that(&has_hidden_component(&world, entity)).is_false();
}

#[test]
fn test_blinking_system_normal_interval_multiple_intervals() {
    let mut world = create_blinking_test_world();
    let entity = spawn_blinking_entity(&mut world, 3);

    // Run system with 7 ticks (2 complete intervals + 1 remainder)
    run_blinking_system(&mut world, 7);

    // Should toggle twice (even number), so back to original state (not hidden)
    assert_that(&has_hidden_component(&world, entity)).is_false();

    // Check that timer was updated to remainder
    let blinking = world.entity(entity).get::<Blinking>().unwrap();
    assert_that(&blinking.tick_timer).is_equal_to(1);
}

#[test]
fn test_blinking_system_normal_interval_odd_intervals() {
    let mut world = create_blinking_test_world();
    let entity = spawn_blinking_entity(&mut world, 2);

    // Run system with 5 ticks (2 complete intervals + 1 remainder)
    run_blinking_system(&mut world, 5);

    // Should toggle twice (even number), so back to original state (not hidden)
    assert_that(&has_hidden_component(&world, entity)).is_false();

    // Check that timer was updated to remainder
    let blinking = world.entity(entity).get::<Blinking>().unwrap();
    assert_that(&blinking.tick_timer).is_equal_to(1);
}

#[test]
fn test_blinking_system_zero_interval_with_ticks() {
    let mut world = create_blinking_test_world();
    let entity = spawn_blinking_entity(&mut world, 0);

    // Run system with any positive ticks
    run_blinking_system(&mut world, 1);

    // Entity should be hidden immediately
    assert_that(&has_hidden_component(&world, entity)).is_true();
}

#[test]
fn test_blinking_system_zero_interval_no_ticks() {
    let mut world = create_blinking_test_world();
    let entity = spawn_blinking_entity(&mut world, 0);

    // Run system with 0 ticks
    run_blinking_system(&mut world, 0);

    // Entity should not be hidden (no time passed)
    assert_that(&has_hidden_component(&world, entity)).is_false();
}

#[test]
fn test_blinking_system_zero_interval_toggle_back() {
    let mut world = create_blinking_test_world();
    let entity = spawn_hidden_blinking_entity(&mut world, 0);

    // Run system with any positive ticks
    run_blinking_system(&mut world, 1);

    // Entity should be unhidden
    assert_that(&has_hidden_component(&world, entity)).is_false();
}

#[test]
fn test_blinking_system_frozen_entity_unhidden() {
    let mut world = create_blinking_test_world();
    let entity = spawn_frozen_hidden_blinking_entity(&mut world, 5);

    // Run system with ticks
    run_blinking_system(&mut world, 10);

    // Frozen entity should be unhidden and stay unhidden
    assert_that(&has_hidden_component(&world, entity)).is_false();
    assert_that(&has_frozen_component(&world, entity)).is_true();
}

#[test]
fn test_blinking_system_frozen_entity_no_blinking() {
    let mut world = create_blinking_test_world();
    let entity = spawn_frozen_blinking_entity(&mut world, 5);

    // Run system with ticks
    run_blinking_system(&mut world, 10);

    // Frozen entity should not be hidden (blinking disabled)
    assert_that(&has_hidden_component(&world, entity)).is_false();
    assert_that(&has_frozen_component(&world, entity)).is_true();
}

#[test]
fn test_blinking_system_frozen_entity_timer_not_updated() {
    let mut world = create_blinking_test_world();
    let entity = spawn_frozen_blinking_entity(&mut world, 5);

    // Run system with ticks
    run_blinking_system(&mut world, 10);

    // Timer should not be updated for frozen entities
    let blinking = world.entity(entity).get::<Blinking>().unwrap();
    assert_that(&blinking.tick_timer).is_equal_to(0);
}

#[test]
fn test_blinking_system_entity_without_renderable_ignored() {
    let mut world = create_blinking_test_world();

    // Spawn entity with only Blinking component (no Renderable)
    let entity = world.spawn(Blinking::new(5)).id();

    // Run system
    run_blinking_system(&mut world, 10);

    // Entity should not be affected (not in query)
    assert_that(&has_hidden_component(&world, entity)).is_false();
}

#[test]
fn test_blinking_system_entity_without_blinking_ignored() {
    let mut world = create_blinking_test_world();

    // Spawn entity with only Renderable component (no Blinking)
    let entity = world
        .spawn(Renderable {
            sprite: common::mock_atlas_tile(1),
            layer: 0,
        })
        .id();

    // Run system
    run_blinking_system(&mut world, 10);

    // Entity should not be affected (not in query)
    assert_that(&has_hidden_component(&world, entity)).is_false();
}

#[test]
fn test_blinking_system_large_interval() {
    let mut world = create_blinking_test_world();
    let entity = spawn_blinking_entity(&mut world, 1000);

    // Run system with 500 ticks (less than interval)
    run_blinking_system(&mut world, 500);

    // Entity should not be hidden yet
    assert_that(&has_hidden_component(&world, entity)).is_false();

    // Check that timer was updated
    let blinking = world.entity(entity).get::<Blinking>().unwrap();
    assert_that(&blinking.tick_timer).is_equal_to(500);
}

#[test]
fn test_blinking_system_very_small_interval() {
    let mut world = create_blinking_test_world();
    let entity = spawn_blinking_entity(&mut world, 1);

    // Run system with 1 tick
    run_blinking_system(&mut world, 1);

    // Entity should be hidden
    assert_that(&has_hidden_component(&world, entity)).is_true();

    // Run system with another 1 tick
    run_blinking_system(&mut world, 1);

    // Entity should be unhidden
    assert_that(&has_hidden_component(&world, entity)).is_false();
}
