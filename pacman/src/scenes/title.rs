//! The Title scene: a minimal press-to-start gate shown at boot.

use bevy_ecs::event::EventReader;
use bevy_ecs::system::ResMut;
use bevy_ecs::world::World;

use crate::error::GameResult;
use crate::events::GameEvent;

use super::{Scene, SceneHandler, SceneManager};

/// The Title screen. It owns no entities and renders no bespoke content yet
/// (deferred); it exists to prove the scene routing -- boot lands here, and the
/// first player input hands off to [`Scene::Gameplay`].
pub struct TitleScene;

impl SceneHandler for TitleScene {
    /// Entering the Title spawns nothing -- it is a passive screen awaiting input.
    fn on_enter(&self, _world: &mut World) -> GameResult<()> {
        Ok(())
    }

    /// The Title owns no entities, so there is nothing to tear down on exit.
    fn on_exit(&self, _world: &mut World) {}
}

/// Hands off to gameplay on the first input while the Title is active. Any bound
/// key press surfaces as a [`GameEvent`], which is enough to start; on the web the
/// JS "Click to Start" overlay instead calls the `start_game` FFI, which queues
/// gameplay directly. The schedule gates this system to the Title scene.
pub fn title_input_system(mut events: EventReader<GameEvent>, mut scenes: ResMut<SceneManager>) {
    if events.read().next().is_some() {
        scenes.request(Scene::Gameplay);
    }
}
