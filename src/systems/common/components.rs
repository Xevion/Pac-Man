use bevy_ecs::{component::Component, resource::Resource};

use crate::map::graph::TraversalFlags;

/// A tag component denoting the type of entity.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntityType {
    Player,
    Ghost,
    Pellet,
    PowerPellet,
    Fruit(crate::texture::sprites::FruitSprite),
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
            EntityType::Fruit(fruit_type) => Some(fruit_type.score_value()),
            _ => None,
        }
    }

    pub fn is_collectible(&self) -> bool {
        matches!(self, EntityType::Pellet | EntityType::PowerPellet | EntityType::Fruit(_))
    }
}

#[derive(Resource)]
pub struct GlobalState {
    pub exit: bool,
}

#[derive(Resource)]
pub struct ScoreResource(pub u32);

#[derive(Resource)]
pub struct DeltaTime {
    /// Floating-point delta time in seconds
    pub seconds: f32,
    /// Integer tick delta (usually 1, but can be different for testing)
    pub ticks: u32,
}

#[allow(dead_code)]
impl DeltaTime {
    /// Creates a new DeltaTime from a floating-point delta time in seconds
    ///
    /// While this method exists as a helper, it does not mean that seconds and ticks are interchangeable.
    pub fn from_seconds(seconds: f32) -> Self {
        Self {
            seconds,
            ticks: (seconds * 60.0).round() as u32,
        }
    }

    /// Creates a new DeltaTime from an integer tick delta
    ///
    /// While this method exists as a helper, it does not mean that seconds and ticks are interchangeable.
    pub fn from_ticks(ticks: u32) -> Self {
        Self {
            seconds: ticks as f32 / 60.0,
            ticks,
        }
    }
}

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

/// Component for HUD life sprite entities.
/// Each life sprite entity has an index indicating its position from left to right (0, 1, 2, etc.).
/// This mostly functions as a tag component for sprites.
#[derive(Component, Debug, Clone, Copy)]
pub struct PlayerLife {
    pub index: u32,
}
