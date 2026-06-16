//! The Gameplay scene: the playable maze of player, ghosts, and collectibles.

use bevy_ecs::world::World;

use crate::error::GameResult;
use crate::game::spawning::{despawn_gameplay, spawn_gameplay};

use super::SceneHandler;

/// The playable maze. Its lifecycle delegates to the spawning module, which owns
/// the actual entity population and teardown contract.
pub struct GameplayScene;

impl SceneHandler for GameplayScene {
    /// Entering gameplay spawns a fresh scene at level 1. Score and lives carry
    /// over in the session; level progression and resets are wired in a later phase.
    fn on_enter(&self, world: &mut World) -> GameResult<()> {
        spawn_gameplay(world, 1)
    }

    /// Leaving gameplay despawns every scene entity and clears any dangling state.
    fn on_exit(&self, world: &mut World) {
        despawn_gameplay(world);
    }
}
