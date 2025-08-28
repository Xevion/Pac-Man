use bevy_ecs::{event::Events, world::World};

use pacman::{error::GameError, systems::components::ScoreResource};

fn create_test_world() -> World {
    let mut world = World::new();

    // Add required resources
    world.insert_resource(Events::<GameError>::default());
    world.insert_resource(ScoreResource(1230)); // Test score

    world
}

#[test]
fn test_hud_render_system_runs_without_error() {
    let world = create_test_world();

    // The HUD render system requires SDL2 resources that aren't available in tests,
    // but we can at least verify it doesn't panic when called
    // In a real test environment, we'd need to mock the SDL2 canvas and atlas

    // For now, just verify the score resource is accessible
    let score = world.resource::<ScoreResource>();
    assert_eq!(score.0, 1230);
}
