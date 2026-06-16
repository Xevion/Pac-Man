//! The Gameplay scene: the playable maze of player, ghosts, and collectibles.

use bevy_ecs::schedule::{IntoScheduleConfigs, Schedule};
use bevy_ecs::world::World;

use crate::error::GameResult;
use crate::game::spawning::{despawn_gameplay, spawn_gameplay};
use crate::systems::input::{InputSet, InputSource};
use crate::systems::profiling::profile;
use crate::systems::state::handle_pause_command;

use super::{in_scene, Scene, SceneHandler};

/// The playable maze. Its lifecycle delegates to the spawning module, which owns
/// the actual entity population and teardown contract.
pub struct GameplayScene;

impl SceneHandler for GameplayScene {
    /// Entering gameplay hands control to the human and spawns a fresh scene at
    /// level 1. Score and lives carry over in the session; level progression and
    /// resets are wired in a later phase.
    fn on_enter(&self, world: &mut World) -> GameResult<()> {
        world.insert_resource(InputSource::Human);
        spawn_gameplay(world, 1)
    }

    /// Leaving gameplay despawns every scene entity and clears any dangling state.
    fn on_exit(&self, world: &mut World) {
        despawn_gameplay(world);
    }

    /// Pause is a Gameplay-only concept: gating the toggle here keeps Escape on the Title
    /// (or during attract) from arming a pause that would carry into the next game. Runs
    /// in the React phase so it sees this frame's `GameEvent`s.
    fn register(&self, schedule: &mut Schedule) {
        schedule.add_systems(
            profile("input", handle_pause_command)
                .run_if(in_scene(Scene::Gameplay))
                .in_set(InputSet::React),
        );
    }
}
