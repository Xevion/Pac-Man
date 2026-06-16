//! Per-game session state.

use bevy_ecs::resource::Resource;

use crate::systems::common::ScoreResource;
use crate::systems::item::PelletCount;

use super::{GameStage, PlayerLives};

/// All state belonging to a single play-through. Created when the Gameplay scene
/// is entered and torn down on exit, so "no game in progress" simply means this
/// resource is absent / freshly reset.
///
/// Holds the low-frequency scalars only. The high-frequency ghost controllers
/// (`GhostModeController`/`GhostHouseController`) stay as separate sibling
/// resources so their per-frame timer mutations don't pollute `Session`'s
/// change-detection, which the render dirty-tracking relies on.
#[derive(Resource, Debug)]
pub struct Session {
    /// Current level -- the single source of truth. The ghost controllers cache
    /// it internally for their per-tick lookups, reconfigured on level change.
    pub level: u8,
    pub score: ScoreResource,
    pub lives: PlayerLives,
    pub pellets: PelletCount,
    /// Whether the opening jingle has played for the current startup sequence.
    pub intro_played: bool,
    /// How many ghosts Pac-Man has eaten during the current fright period. Drives the
    /// 200/400/800/1600 score chain; reset to 0 each time a power pellet is consumed.
    pub ghost_eaten_chain: u8,
    /// The gameplay sub-machine's current stage. Private so every transition goes through
    /// [`Session::set_stage`] -- the single, searchable mutation point for the machine.
    stage: GameStage,
}

impl Session {
    /// Builds a fresh session at the given level.
    pub fn new(level: u8) -> Self {
        Self {
            level,
            score: ScoreResource::default(),
            lives: PlayerLives::default(),
            pellets: PelletCount::default(),
            intro_played: false,
            ghost_eaten_chain: 0,
            stage: GameStage::initial(),
        }
    }

    /// The current gameplay stage. `GameStage` is `Copy`, so this hands back a snapshot.
    pub fn stage(&self) -> GameStage {
        self.stage
    }

    /// The sole way to advance the gameplay sub-machine. Every writer -- the tick-driven
    /// `stage_system`, the ghost-eaten/death observers, and teardown -- routes through here.
    pub fn set_stage(&mut self, stage: GameStage) {
        self.stage = stage;
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new(1)
    }
}
