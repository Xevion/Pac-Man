//! The Title scene: a minimal press-to-start gate shown at boot.

use bevy_ecs::event::EventReader;
use bevy_ecs::system::{Local, Res, ResMut};
use bevy_ecs::world::World;

use crate::error::GameResult;
use crate::events::GameEvent;
use crate::systems::common::DeltaTime;

use super::{Scene, SceneHandler, SceneManager};

/// Seconds the Title waits with no input before falling through to the self-playing
/// attract demo, mirroring the arcade's idle behavior.
const ATTRACT_IDLE_SECS: f32 = 10.0;

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

/// Drives the Title's two exits. The first input hands off to gameplay -- any bound
/// key press surfaces as a [`GameEvent`], which is enough to start (on the web the
/// JS "Click to Start" overlay instead calls the `start_game` FFI, which queues
/// gameplay directly). Absent any input for [`ATTRACT_IDLE_SECS`], it falls through
/// to the self-playing attract demo. The schedule gates this system to the Title
/// scene, so the idle timer only accrues while the Title is up.
pub fn title_input_system(
    mut events: EventReader<GameEvent>,
    time: Res<DeltaTime>,
    mut idle: Local<f32>,
    mut scenes: ResMut<SceneManager>,
) {
    if events.read().next().is_some() {
        *idle = 0.0;
        scenes.request(Scene::Gameplay);
        return;
    }

    *idle += time.seconds;
    if *idle >= ATTRACT_IDLE_SECS {
        scenes.request(Scene::Attract);
    }
}
