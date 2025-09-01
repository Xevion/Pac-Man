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
use micromap::Map;

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

/// A component for entities that have a directional animated texture.
#[derive(Component, Clone, Default)]
pub struct DirectionalAnimated {
    pub textures: [Option<AnimatedTexture>; 4],
    pub stopped_textures: [Option<AnimatedTexture>; 4],
}

impl DirectionalAnimated {
    pub fn from_animation(animation: AnimatedTexture) -> Self {
        // Create 4 copies of the animation - necessary for independent state per direction
        // This is initialization-time only, so the cloning cost is acceptable
        Self {
            textures: [
                Some(animation.clone()),
                Some(animation.clone()),
                Some(animation.clone()),
                Some(animation.clone()),
            ],
            stopped_textures: [
                Some(animation.clone()),
                Some(animation.clone()),
                Some(animation.clone()),
                Some(animation),
            ],
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

/// Component for ghosts that are vulnerable to Pac-Man
#[derive(Component, Debug, Clone, Copy)]
pub struct Vulnerable {
    pub remaining_ticks: u32,
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

/// A complete set of animations for a ghost in different behavioral states.
///
/// Each ghost maintains animations mapped by their current gameplay state.
/// The animation system automatically switches between these states based on
/// the presence of `Vulnerable` and `Eaten` components on the ghost entity.
#[derive(Component, Clone)]
pub struct GhostAnimationSet {
    pub animations: Map<GhostAnimation, DirectionalAnimated, 4>,
}

impl GhostAnimationSet {
    /// Creates a new GhostAnimationSet with the provided animations.
    pub fn new(
        normal: DirectionalAnimated,
        frightened: DirectionalAnimated,
        frightened_flashing: DirectionalAnimated,
        eyes: DirectionalAnimated,
    ) -> Self {
        let mut animations = Map::new();
        animations.insert(GhostAnimation::Normal, normal);
        animations.insert(GhostAnimation::Frightened { flash: false }, frightened);
        animations.insert(GhostAnimation::Frightened { flash: true }, frightened_flashing);
        animations.insert(GhostAnimation::Eyes, eyes);
        Self { animations }
    }

    /// Gets the animation for the specified ghost animation state.
    pub fn get(&self, animation: GhostAnimation) -> Option<&DirectionalAnimated> {
        self.animations.get(&animation)
    }

    /// Gets the normal animation state.
    pub fn normal(&self) -> Option<&DirectionalAnimated> {
        self.get(GhostAnimation::Normal)
    }

    /// Gets the frightened animation state (non-flashing).
    pub fn frightened(&self) -> Option<&DirectionalAnimated> {
        self.get(GhostAnimation::Frightened { flash: false })
    }

    /// Gets the frightened flashing animation state.
    pub fn frightened_flashing(&self) -> Option<&DirectionalAnimated> {
        self.get(GhostAnimation::Frightened { flash: true })
    }

    /// Gets the eyes animation state (for eaten ghosts).
    pub fn eyes(&self) -> Option<&DirectionalAnimated> {
        self.get(GhostAnimation::Eyes)
    }
}

/// Global resource containing pre-loaded animation sets for all ghost types.
///
/// This resource is initialized once during game startup and provides O(1) access
/// to animation sets for each ghost type. The animation system uses this resource
/// to efficiently switch between different ghost states without runtime asset loading.
///
/// The HashMap is keyed by `Ghost` enum variants (Blinky, Pinky, Inky, Clyde) and
/// contains complete animation sets mapped by GhostAnimation states.
#[derive(Resource)]
pub struct GhostAnimations(pub std::collections::HashMap<Ghost, GhostAnimationSet>);

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
