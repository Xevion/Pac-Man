//! Tests for the Title scene's input handling: what starts the game vs. what doesn't.
//!
//! The Title starts gameplay only on a genuine intent to play -- a movement key or a
//! click/tap -- so meta commands (notably Escape/pause) neither start the game nor
//! leak a pause into it.

use bevy_ecs::event::EventRegistry;
use bevy_ecs::system::RunSystemOnce;
use bevy_ecs::world::World;
use glam::Vec2;

use pacman::events::{GameCommand, GameEvent};
use pacman::map::direction::Direction;
use pacman::scenes::{title_input_system, Scene, SceneManager};
use pacman::systems::common::DeltaTime;
use pacman::systems::input::{TouchData, TouchState};
use speculoos::prelude::*;

/// A minimal world holding exactly what `title_input_system` reads, sitting on the Title.
fn title_world() -> World {
    let mut world = World::new();
    EventRegistry::register_event::<GameEvent>(&mut world);
    world.insert_resource(TouchState::default());
    world.insert_resource(DeltaTime { seconds: 0.0, ticks: 0 });
    world.insert_resource(SceneManager::new(Scene::Title));
    world
}

/// A movement key is a play intent: it queues the Gameplay transition.
#[test]
fn title_starts_on_movement_input() {
    let mut world = title_world();
    world.send_event(GameEvent::Command(GameCommand::MovePlayer(Direction::Up)));
    world.run_system_once(title_input_system).unwrap();
    assert_that(&world.resource::<SceneManager>().pending()).is_equal_to(Some(Scene::Gameplay));
}

/// A click/tap (a touch with no command) also starts -- the desktop click-to-start path.
#[test]
fn title_starts_on_tap() {
    let mut world = title_world();
    world.resource_mut::<TouchState>().active_touch = Some(TouchData::new(0, Vec2::ZERO));
    world.run_system_once(title_input_system).unwrap();
    assert_that(&world.resource::<SceneManager>().pending()).is_equal_to(Some(Scene::Gameplay));
}

/// A meta command (pause) must not start the game -- otherwise Escape would both start
/// and immediately toggle pause.
#[test]
fn title_ignores_meta_command() {
    let mut world = title_world();
    world.send_event(GameEvent::Command(GameCommand::TogglePause));
    world.run_system_once(title_input_system).unwrap();
    assert_that(&world.resource::<SceneManager>().pending()).is_equal_to(None);
}
