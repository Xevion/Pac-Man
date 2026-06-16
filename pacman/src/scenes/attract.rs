//! The Attract scene: a self-playing demo of gameplay.
//!
//! Mechanically it *is* the Gameplay scene -- same maze, ghosts, and collectibles,
//! spawned and torn down through the exact same contract. The only difference is the
//! input source: entering flips it to [`InputSource::Ai`] so the stub AI drives
//! Pac-Man, and leaving restores [`InputSource::Human`].

use bevy_ecs::world::World;

use crate::error::GameResult;
use crate::game::spawning::{despawn_gameplay, spawn_gameplay};
use crate::systems::input::InputSource;

use super::SceneHandler;

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
