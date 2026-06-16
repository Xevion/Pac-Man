//! Tests for closing the attract loop: any human input starts a real game, and the
//! READY!/GAME OVER overlays follow the live simulation (attract included).

use bevy_ecs::system::RunSystemOnce;
use bevy_ecs::world::World;

use pacman::scenes::{attract_input_system, in_simulation, Scene, SceneManager};
use pacman::systems::input::HumanInput;
use speculoos::prelude::*;

/// Any human input during attract queues the transition to a real game.
#[test]
fn attract_starts_on_any_human_input() {
    let mut world = World::new();
    world.insert_resource(SceneManager::new(Scene::Attract));
    world.insert_resource(HumanInput { active: true });

    world.run_system_once(attract_input_system).unwrap();

    assert_that(&world.resource::<SceneManager>().pending()).is_equal_to(Some(Scene::Gameplay));
}

/// With no human input, the demo keeps playing -- no transition is queued.
#[test]
fn attract_keeps_playing_without_human_input() {
    let mut world = World::new();
    world.insert_resource(SceneManager::new(Scene::Attract));
    world.insert_resource(HumanInput { active: false });

    world.run_system_once(attract_input_system).unwrap();

    assert_that(&world.resource::<SceneManager>().pending()).is_equal_to(None);
}

/// The overlay-gating run-condition treats Attract as a live simulation (so READY!/GAME
/// OVER render there) but excludes the Title's empty maze.
#[test]
fn simulation_gate_includes_attract_excludes_title() {
    let mut world = World::new();

    world.insert_resource(SceneManager::new(Scene::Attract));
    assert_that(&world.run_system_once(in_simulation).unwrap()).is_true();

    world.insert_resource(SceneManager::new(Scene::Gameplay));
    assert_that(&world.run_system_once(in_simulation).unwrap()).is_true();

    world.insert_resource(SceneManager::new(Scene::Title));
    assert_that(&world.run_system_once(in_simulation).unwrap()).is_false();
}
