//! Ghost state machine tracking house status, activity, and frightened state.

use bevy_ecs::component::Component;

/// Per-ghost state machine tracking house status, activity, and frightened state
#[derive(Component, Debug, Clone, PartialEq)]
pub enum GhostState {
    /// Inside the ghost house, bouncing up/down waiting to exit
    InHouse {
        position: HousePosition,
        bounce: BounceDirection,
    },

    /// Moving from house interior through the door
    Exiting {
        /// Progress 0.0 (at center) to 1.0 (at entrance node)
        progress: f32,
    },

    /// Active in the maze, following scatter/chase/frightened logic
    Active {
        /// If Some, ghost is frightened (overrides global mode)
        /// If None, follows GhostModeController's current mode
        frightened: Option<FrightenedData>,
    },

    /// Eyes returning to ghost house after being eaten
    Eyes,

    /// Inside house after returning as eyes, waiting before re-exit
    Reviving { remaining_ticks: u32 },
}

impl Default for GhostState {
    fn default() -> Self {
        Self::Active { frightened: None }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // Variants needed once ghost house spawning uses correct per-ghost positions
pub enum HousePosition {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // Down used once ghost house bounce animation is wired up
pub enum BounceDirection {
    Up,
    Down,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FrightenedData {
    pub remaining_ticks: u32,
    pub flashing: bool,
    pub flash_timer: u32,
}

impl FrightenedData {
    pub fn new(total_ticks: u32, flash_start_ticks: u32) -> Self {
        Self {
            remaining_ticks: total_ticks,
            flashing: false,
            flash_timer: flash_start_ticks,
        }
    }

    /// Returns true if frightened mode ended
    pub fn tick(&mut self) -> bool {
        if self.remaining_ticks == 0 {
            return true;
        }
        self.remaining_ticks -= 1;

        if self.flash_timer > 0 {
            self.flash_timer -= 1;
            if self.flash_timer == 0 {
                self.flashing = true;
            }
        }
        false
    }
}

impl GhostState {
    /// Returns the visual animation state for rendering
    pub fn animation_state(&self) -> GhostAnimationState {
        match self {
            Self::InHouse { .. } | Self::Exiting { .. } => GhostAnimationState::Normal,
            Self::Active { frightened: None } => GhostAnimationState::Normal,
            Self::Active { frightened: Some(f) } => GhostAnimationState::Frightened { flash: f.flashing },
            Self::Eyes | Self::Reviving { .. } => GhostAnimationState::Eyes,
        }
    }

    /// Returns true if the ghost is inside the house (any house-related state)
    pub fn is_in_house(&self) -> bool {
        matches!(self, Self::InHouse { .. } | Self::Exiting { .. } | Self::Reviving { .. })
    }

    /// Returns true if the ghost is actively chasing/scattering in the maze
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Active { .. })
    }

    /// Returns true if ghost is frightened
    pub fn is_frightened(&self) -> bool {
        matches!(self, Self::Active { frightened: Some(_) })
    }

    /// Ticks the ghost state, handling internal transitions
    /// Returns true if the state changed
    pub fn tick(&mut self) -> bool {
        match self {
            Self::Active {
                frightened: Some(ref mut f),
            } => {
                if f.tick() {
                    // Frightened mode ended
                    *self = Self::Active { frightened: None };
                    true
                } else {
                    false
                }
            }
            Self::Reviving { remaining_ticks } => {
                if *remaining_ticks == 0 {
                    false
                } else {
                    *remaining_ticks -= 1;
                    *remaining_ticks == 0
                }
            }
            _ => false,
        }
    }
}

/// Animation state (mapped from GhostState for rendering)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GhostAnimationState {
    Normal,
    Frightened { flash: bool },
    Eyes,
}
