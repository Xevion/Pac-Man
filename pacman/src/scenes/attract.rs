//! The Attract scene: a self-playing demo of gameplay.
//!
//! Mechanically it *is* the Gameplay scene -- same maze, ghosts, and collectibles,
//! spawned and torn down through the exact same contract. The only difference is the
//! input source: entering flips it to [`InputSource::Ai`] so the stub AI drives
//! Pac-Man, and leaving restores [`InputSource::Human`].

use bevy_ecs::system::{Res, ResMut};
use bevy_ecs::world::World;

use crate::error::GameResult;
use crate::game::spawning::{despawn_gameplay, spawn_gameplay};
use crate::systems::input::{HumanInput, InputSource};

use super::{Scene, SceneHandler, SceneManager};

/// The self-playing attract demo. Reuses the gameplay population under AI control.
pub struct AttractScene;

impl SceneHandler for AttractScene {
    /// Hand control to the AI, then spawn the gameplay scene at level 1.
    fn on_enter(&self, world: &mut World) -> GameResult<()> {
        world.insert_resource(InputSource::Ai);
        spawn_gameplay(world, 1)
    }

    /// Tear the demo down and return control to the human for whatever comes next.
    fn on_exit(&self, world: &mut World) {
        despawn_gameplay(world);
        world.insert_resource(InputSource::Human);
    }
}

/// While the attract demo plays, any human input -- a key or a click/tap -- starts a
/// real game. AI is the movement producer during attract, so human movement commands
/// are suppressed before they surface as events; this instead keys off the
/// `InputSource`-independent [`HumanInput`] pulse (set from raw SDL input). The schedule
/// gates this to the Attract scene, so it only fires while the demo is up.
pub fn attract_input_system(human_input: Res<HumanInput>, mut scenes: ResMut<SceneManager>) {
    if human_input.active {
        scenes.request(Scene::Gameplay);
    }
}
