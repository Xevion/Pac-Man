use bevy_ecs::{bundle::Bundle, component::Component, resource::Resource};
use bitflags::bitflags;

use crate::{
    map::graph::TraversalFlags,
    systems::{
        movement::{BufferedDirection, Position, Velocity},
        Collider, GhostCollider, ItemCollider, PacmanCollider,
    },
    texture::{animated::AnimatedTexture, sprite::AtlasTile},
};

/// A tag component for entities that are controlled by the player.
#[derive(Default, Component)]
pub struct PlayerControlled;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
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

/// A component for entities that have a directional animated texture.
#[derive(Component)]
pub struct DirectionalAnimated {
    pub textures: [Option<AnimatedTexture>; 4],
    pub stopped_textures: [Option<AnimatedTexture>; 4],
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

/// Component for ghosts that are vulnerable to Pac-Man
#[derive(Component, Debug, Clone, Copy)]
pub struct Vulnerable {
    pub remaining_ticks: u32,
}

#[derive(Bundle)]
pub struct PlayerBundle {
    pub player: PlayerControlled,
    pub position: Position,
    pub velocity: Velocity,
    pub buffered_direction: BufferedDirection,
    pub sprite: Renderable,
    pub directional_animated: DirectionalAnimated,
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
    pub directional_animated: DirectionalAnimated,
    pub entity_type: EntityType,
    pub collider: Collider,
    pub ghost_collider: GhostCollider,
}
