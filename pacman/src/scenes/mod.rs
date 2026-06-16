//! Scene layer: the top-altitude identity of what the game is currently showing.
//!
//! Every entity spawned for a scene is tagged [`SceneOwned`] so that leaving the
//! scene can despawn exactly its own entities and nothing else. The full
//! `SceneManager`/router and additional scenes land in a later phase; for now the
//! only scene is gameplay, spawned and torn down by `game::spawn_gameplay` /
//! `game::despawn_gameplay`.

use bevy_ecs::component::Component;

/// The top-level screen the game is currently presenting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scene {
    /// The playable maze: player, ghosts, and collectibles.
    Gameplay,
}

/// Tags an entity as belonging to a particular [`Scene`]. When that scene is torn
/// down, every entity carrying this component is despawned together, so no
/// gameplay entity outlives the scene that owns it.
#[derive(Component, Debug, Clone, Copy)]
pub struct SceneOwned(pub Scene);
