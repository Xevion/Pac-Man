//! Ghost AI system with personality-based targeting and state management.
//!
//! This module implements the classic Pac-Man ghost behavior including:
//! - Scatter/Chase/Frightened mode cycling
//! - Per-ghost targeting personalities (Blinky, Pinky, Inky, Clyde)
//! - Ghost house management
//! - Special behaviors (Elroy mode, tunnel slowdown, red zones)

use bevy_ecs::component::Component;
use std::collections::HashMap;

pub mod elroy;
pub mod house;
pub mod mode;
pub mod movement;
pub mod personality;
pub mod state;
pub mod systems;
pub mod targeting;

pub use elroy::{BlinkyMarker, Elroy};
pub use house::GhostHouseController;
pub use mode::{GhostModeController, ScatterChaseMode};
pub use movement::{ghost_movement_system, ghost_targeting_system, GhostSpeedConfig, GhostTarget, TunnelNodes};
pub use state::{FrightenedData, GhostAnimationState, GhostState};
pub use systems::{eaten_ghost_system, elroy_system, ghost_house_system, ghost_mode_tick_system, ghost_state_system};
pub use targeting::RedZoneNodes;

use crate::systems::{DirectionalAnimation, LinearAnimation};

/// Ghost personality type -- determines targeting behavior
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GhostType {
    Blinky,
    Pinky,
    Inky,
    Clyde,
}

impl GhostType {
    /// Returns the ghost type name for atlas lookups
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Blinky => "blinky",
            Self::Pinky => "pinky",
            Self::Inky => "inky",
            Self::Clyde => "clyde",
        }
    }

    /// Base speed multiplier (all ghosts move at same base speed in authentic Pac-Man).
    /// Actual speed varies by level, not ghost type.
    pub fn base_speed(self) -> f32 {
        1.0
    }
}

/// Global resource containing pre-loaded animation sets for all ghost types
#[derive(bevy_ecs::resource::Resource)]
pub struct GhostAnimations {
    pub normal: HashMap<GhostType, DirectionalAnimation>,
    pub eyes: DirectionalAnimation,
    pub frightened: LinearAnimation,
    pub frightened_flashing: LinearAnimation,
}

impl GhostAnimations {
    pub fn new(
        normal: HashMap<GhostType, DirectionalAnimation>,
        eyes: DirectionalAnimation,
        frightened: LinearAnimation,
        frightened_flashing: LinearAnimation,
    ) -> Self {
        Self {
            normal,
            eyes,
            frightened,
            frightened_flashing,
        }
    }

    pub fn get_normal(&self, ghost_type: &GhostType) -> Option<&DirectionalAnimation> {
        self.normal.get(ghost_type)
    }

    pub fn eyes(&self) -> &DirectionalAnimation {
        &self.eyes
    }

    pub fn frightened(&self, flash: bool) -> &LinearAnimation {
        if flash {
            &self.frightened_flashing
        } else {
            &self.frightened
        }
    }
}

/// Component to track the last animation state for efficient change detection
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct LastAnimationState(pub GhostAnimationState);
