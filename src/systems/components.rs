use std::collections::HashMap;

use bevy_ecs::{bundle::Bundle, component::Component, resource::Resource};
use bitflags::bitflags;

use crate::{
    map::graph::TraversalFlags,
    systems::{
        movement::{BufferedDirection, Position, Velocity},
        Collider, GhostCollider, ItemCollider, PacmanCollider,
    },
    texture::{
        animated::{DirectionalTiles, TileSequence},
        sprite::AtlasTile,
    },
};

/// A tag component for entities that are controlled by the player.
#[derive(Default, Component)]
pub struct PlayerControlled;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ghost {
    Blinky,
    Pinky,
    Inky,
    Clyde,
}

impl Ghost {
    /// Returns the ghost type name for atlas lookups.
    pub fn as_str(self) -> &'static str {
        match self {
            Ghost::Blinky => "blinky",
            Ghost::Pinky => "pinky",
            Ghost::Inky => "inky",
            Ghost::Clyde => "clyde",
        }
    }

    /// Returns the base movement speed for this ghost type.
    pub fn base_speed(self) -> f32 {
        match self {
            Ghost::Blinky => 1.0,
            Ghost::Pinky => 0.95,
            Ghost::Inky => 0.9,
            Ghost::Clyde => 0.85,
        }
    }

    /// Returns the ghost's color for debug rendering.
    #[allow(dead_code)]
    pub fn debug_color(&self) -> sdl2::pixels::Color {
        match self {
            Ghost::Blinky => sdl2::pixels::Color::RGB(255, 0, 0),    // Red
            Ghost::Pinky => sdl2::pixels::Color::RGB(255, 182, 255), // Pink
            Ghost::Inky => sdl2::pixels::Color::RGB(0, 255, 255),    // Cyan
            Ghost::Clyde => sdl2::pixels::Color::RGB(255, 182, 85),  // Orange
        }
    }
}

/// A tag component denoting the type of entity.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntityType {
    Player,
    Ghost,
    Pellet,
    PowerPellet,
}

impl EntityType {
    /// Returns the traversal flags for this entity type.
    pub fn traversal_flags(&self) -> TraversalFlags {
        match self {
            EntityType::Player => TraversalFlags::PACMAN,
            EntityType::Ghost => TraversalFlags::GHOST,
            _ => TraversalFlags::empty(), // Static entities don't traverse
        }
    }
    pub fn score_value(&self) -> Option<u32> {
        match self {
            EntityType::Pellet => Some(10),
            EntityType::PowerPellet => Some(50),
            _ => None,
        }
    }

    pub fn is_collectible(&self) -> bool {
        matches!(self, EntityType::Pellet | EntityType::PowerPellet)
    }
}

/// A component for entities that have a sprite, with a layer for ordering.
///
/// This is intended to be modified by other entities allowing animation.
#[derive(Component)]
pub struct Renderable {
    pub sprite: AtlasTile,
    pub layer: u8,
}

/// Directional animation component with shared timing across all directions
#[derive(Component, Clone, Copy)]
pub struct DirectionalAnimation {
    pub moving_tiles: DirectionalTiles,
    pub stopped_tiles: DirectionalTiles,
    pub current_frame: usize,
    pub time_bank: u16,
    pub frame_duration: u16,
}

impl DirectionalAnimation {
    /// Creates a new directional animation with the given tiles and frame duration
    pub fn new(moving_tiles: DirectionalTiles, stopped_tiles: DirectionalTiles, frame_duration: u16) -> Self {
        Self {
            moving_tiles,
            stopped_tiles,
            current_frame: 0,
            time_bank: 0,
            frame_duration,
        }
    }
}

/// Linear animation component for non-directional animations (frightened ghosts)
#[derive(Component, Clone, Copy)]
pub struct LinearAnimation {
    pub tiles: TileSequence,
    pub current_frame: usize,
    pub time_bank: u16,
    pub frame_duration: u16,
}

impl LinearAnimation {
    /// Creates a new linear animation with the given tiles and frame duration
    pub fn new(tiles: TileSequence, frame_duration: u16) -> Self {
        Self {
            tiles,
            current_frame: 0,
            time_bank: 0,
            frame_duration,
        }
    }
}

bitflags! {
    #[derive(Component, Default, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct CollisionLayer: u8 {
        const PACMAN = 1 << 0;
        const GHOST = 1 << 1;
        const ITEM = 1 << 2;
    }
}

#[derive(Resource)]
pub struct GlobalState {
    pub exit: bool,
}

#[derive(Resource)]
pub struct ScoreResource(pub u32);

#[derive(Resource)]
pub struct DeltaTime(pub f32);

/// Movement modifiers that can affect Pac-Man's speed or handling.
#[derive(Component, Debug, Clone, Copy)]
pub struct MovementModifiers {
    /// Multiplier applied to base speed (e.g., tunnels)
    pub speed_multiplier: f32,
    /// True when currently in a tunnel slowdown region
    pub tunnel_slowdown_active: bool,
}

impl Default for MovementModifiers {
    fn default() -> Self {
        Self {
            speed_multiplier: 1.0,
            tunnel_slowdown_active: false,
        }
    }
}

/// Tag component for entities that should be frozen during startup
#[derive(Component, Debug, Clone, Copy)]
pub struct Frozen;

/// Tag component for eaten ghosts
#[derive(Component, Debug, Clone, Copy)]
pub struct Eaten;

#[derive(Component, Debug, Clone, Copy)]
pub enum GhostState {
    /// Normal ghost behavior - chasing Pac-Man
    Normal,
    /// Frightened state after power pellet - ghost can be eaten
    Frightened {
        remaining_ticks: u32,
        flash: bool,
        remaining_flash_ticks: u32,
    },
    /// Eyes state - ghost has been eaten and is returning to ghost house
    Eyes,
}

/// Component to track the last animation state for efficient change detection
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct LastAnimationState(pub GhostAnimation);

impl GhostState {
    /// Creates a new frightened state with the specified duration
    pub fn new_frightened(total_ticks: u32, flash_start_ticks: u32) -> Self {
        Self::Frightened {
            remaining_ticks: total_ticks,
            flash: false,
            remaining_flash_ticks: flash_start_ticks, // Time until flashing starts
        }
    }

    /// Ticks the ghost state, returning true if the state changed.
    pub fn tick(&mut self) -> bool {
        if let GhostState::Frightened {
            remaining_ticks,
            flash,
            remaining_flash_ticks,
        } = self
        {
            // Transition out of frightened state
            if *remaining_ticks == 0 {
                *self = GhostState::Normal;
                return true;
            }

            *remaining_ticks -= 1;

            if *remaining_flash_ticks > 0 {
                *remaining_flash_ticks = remaining_flash_ticks.saturating_sub(1);
                if *remaining_flash_ticks == 0 {
                    *flash = true;
                    true
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Returns the appropriate animation state for this ghost state
    pub fn animation_state(&self) -> GhostAnimation {
        match self {
            GhostState::Normal => GhostAnimation::Normal,
            GhostState::Eyes => GhostAnimation::Eyes,
            GhostState::Frightened { flash: false, .. } => GhostAnimation::Frightened { flash: false },
            GhostState::Frightened { flash: true, .. } => GhostAnimation::Frightened { flash: true },
        }
    }
}

/// Enumeration of different ghost animation states.
/// Note that this is used in micromap which has a fixed size based on the number of variants,
/// so extending this should be done with caution, and will require updating the micromap's capacity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GhostAnimation {
    /// Normal ghost appearance with directional movement animations
    Normal,
    /// Blue ghost appearance when vulnerable (power pellet active)
    Frightened { flash: bool },
    /// Eyes-only animation when ghost has been consumed by Pac-Man (Eaten state)
    Eyes,
}

/// Global resource containing pre-loaded animation sets for all ghost types.
///
/// This resource is initialized once during game startup and provides O(1) access
/// to animation sets for each ghost type. The animation system uses this resource
/// to efficiently switch between different ghost states without runtime asset loading.
///
/// The HashMap is keyed by `Ghost` enum variants (Blinky, Pinky, Inky, Clyde) and
/// contains the normal directional animation for each ghost type.
#[derive(Resource)]
pub struct GhostAnimations {
    pub normal: HashMap<Ghost, DirectionalAnimation>,
    pub eyes: DirectionalAnimation,
    pub frightened: LinearAnimation,
    pub frightened_flashing: LinearAnimation,
}

impl GhostAnimations {
    /// Creates a new GhostAnimations resource with the provided data.
    pub fn new(
        normal: HashMap<Ghost, DirectionalAnimation>,
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

    /// Gets the normal directional animation for the specified ghost type.
    pub fn get_normal(&self, ghost_type: &Ghost) -> Option<&DirectionalAnimation> {
        self.normal.get(ghost_type)
    }

    /// Gets the eyes animation (shared across all ghosts).
    pub fn eyes(&self) -> &DirectionalAnimation {
        &self.eyes
    }

    /// Gets the frightened animations (shared across all ghosts).
    pub fn frightened(&self, flash: bool) -> &LinearAnimation {
        if flash {
            &self.frightened_flashing
        } else {
            &self.frightened
        }
    }
}

#[derive(Bundle)]
pub struct PlayerBundle {
    pub player: PlayerControlled,
    pub position: Position,
    pub velocity: Velocity,
    pub buffered_direction: BufferedDirection,
    pub sprite: Renderable,
    pub directional_animation: DirectionalAnimation,
    pub entity_type: EntityType,
    pub collider: Collider,
    pub movement_modifiers: MovementModifiers,
    pub pacman_collider: PacmanCollider,
}

#[derive(Bundle)]
pub struct ItemBundle {
    pub position: Position,
    pub sprite: Renderable,
    pub entity_type: EntityType,
    pub collider: Collider,
    pub item_collider: ItemCollider,
}

#[derive(Bundle)]
pub struct GhostBundle {
    pub ghost: Ghost,
    pub position: Position,
    pub velocity: Velocity,
    pub sprite: Renderable,
    pub directional_animation: DirectionalAnimation,
    pub entity_type: EntityType,
    pub collider: Collider,
    pub ghost_collider: GhostCollider,
    pub ghost_state: GhostState,
    pub last_animation_state: LastAnimationState,
}
