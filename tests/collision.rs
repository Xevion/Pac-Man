use bevy_ecs::{entity::Entity, event::Events, system::RunSystemOnce, world::World};

use pacman::{
    error::GameError,
    events::GameEvent,
    map::builder::Map,
    systems::{
        check_collision, collision_system, Collider, EntityType, Ghost, GhostCollider, ItemCollider, PacmanCollider, Position,
    },
};

fn create_test_world() -> World {
    let mut world = World::new();

    // Add required resources
    world.insert_resource(Events::<GameEvent>::default());
    world.insert_resource(Events::<GameError>::default());

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
        .spawn((Position::Stopped { node: 0 }, Collider { size: 10.0 }, PacmanCollider))
        .id()
}

fn spawn_test_item(world: &mut World) -> Entity {
    world
        .spawn((
            Position::Stopped { node: 0 },
            Collider { size: 8.0 },
            ItemCollider,
            EntityType::Pellet,
        ))
        .id()
}

fn spawn_test_ghost(world: &mut World) -> Entity {
    world
        .spawn((
            Position::Stopped { node: 0 },
            Collider { size: 12.0 },
            GhostCollider,
            Ghost::Blinky,
            EntityType::Ghost,
        ))
        .id()
}

fn spawn_test_ghost_at_node(world: &mut World, node: usize) -> Entity {
    world
        .spawn((
            Position::Stopped { node },
            Collider { size: 12.0 },
            GhostCollider,
            Ghost::Blinky,
            EntityType::Ghost,
        ))
        .id()
}

#[test]
fn test_collider_collision_detection() {
    let collider1 = Collider { size: 10.0 };
    let collider2 = Collider { size: 8.0 };

    // Test collision detection
    assert!(collider1.collides_with(collider2.size, 5.0)); // Should collide (distance < 9.0)
    assert!(!collider1.collides_with(collider2.size, 15.0)); // Should not collide (distance > 9.0)
}

#[test]
fn test_check_collision_helper() {
    let map = create_test_map();
    let pos1 = Position::Stopped { node: 0 };
    let pos2 = Position::Stopped { node: 0 }; // Same position
    let collider1 = Collider { size: 10.0 };
    let collider2 = Collider { size: 8.0 };

    // Test collision at same position
    let result = check_collision(&pos1, &collider1, &pos2, &collider2, &map);
    assert!(result.is_ok());
    assert!(result.unwrap()); // Should collide at same position

    // Test collision at different positions
    let pos3 = Position::Stopped { node: 1 }; // Different position
    let result = check_collision(&pos1, &collider1, &pos3, &collider2, &map);
    assert!(result.is_ok());
    // May or may not collide depending on actual node positions
}

#[test]
fn test_collision_system_pacman_item() {
    let mut world = create_test_world();
    let _pacman = spawn_test_pacman(&mut world);
    let _item = spawn_test_item(&mut world);

    // Run collision system - should not panic
    world
        .run_system_once(collision_system)
        .expect("System should run successfully");
}

#[test]
fn test_collision_system_pacman_ghost() {
    let mut world = create_test_world();
    let _pacman = spawn_test_pacman(&mut world);
    let _ghost = spawn_test_ghost(&mut world);

    // Run collision system - should not panic
    world
        .run_system_once(collision_system)
        .expect("System should run successfully");
}

#[test]
fn test_collision_system_no_collision() {
    let mut world = create_test_world();
    let _pacman = spawn_test_pacman(&mut world);
    let _ghost = spawn_test_ghost_at_node(&mut world, 1); // Different node

    // Run collision system - should not panic
    world
        .run_system_once(collision_system)
        .expect("System should run successfully");
}

#[test]
fn test_collision_system_multiple_entities() {
    let mut world = create_test_world();
    let _pacman = spawn_test_pacman(&mut world);
    let _item = spawn_test_item(&mut world);
    let _ghost = spawn_test_ghost(&mut world);

    // Run collision system - should not panic
    world
        .run_system_once(collision_system)
        .expect("System should run successfully");
}
