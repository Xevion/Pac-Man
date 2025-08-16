use bevy_ecs::{bundle::Bundle, component::Component, resource::Resource};
use bitflags::bitflags;

use crate::{
    entity::graph::TraversalFlags,
    systems::movement::{Movable, MovementState, Position},
    texture::{animated::AnimatedTexture, sprite::AtlasTile},
};

/// A tag component for entities that are controlled by the player.
#[derive(Default, Component)]
pub struct PlayerControlled;

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
}

/// A component for entities that have a sprite, with a layer for ordering.
///
/// This is intended to be modified by other entities allowing animation.
#[derive(Component)]
pub struct Renderable {
    pub sprite: AtlasTile,
    pub layer: u8,
    pub visible: bool,
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

#[derive(Component)]
pub struct Collider {
    pub size: f32,
}

/// Marker components for collision filtering optimization
#[derive(Component)]
pub struct PacmanCollider;

#[derive(Component)]
pub struct GhostCollider;

#[derive(Component)]
pub struct ItemCollider;

#[derive(Bundle)]
pub struct PlayerBundle {
    pub player: PlayerControlled,
    pub position: Position,
    pub movement_state: MovementState,
    pub movable: Movable,
    pub sprite: Renderable,
    pub directional_animated: DirectionalAnimated,
    pub entity_type: EntityType,
    pub collider: Collider,
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

#[derive(Resource)]
pub struct GlobalState {
    pub exit: bool,
}

#[derive(Resource)]
pub struct ScoreResource(pub u32);

#[derive(Resource)]
pub struct DeltaTime(pub f32);

#[derive(Resource, Default)]
pub struct RenderDirty(pub bool);

/// Resource for tracking audio state
#[derive(Resource, Debug, Clone, Default)]
pub struct AudioState {
    /// Whether audio is currently muted
    pub muted: bool,
    /// Current sound index for cycling through eat sounds
    pub sound_index: usize,
}
