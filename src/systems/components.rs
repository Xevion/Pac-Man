use bevy_ecs::{bundle::Bundle, component::Component, resource::Resource};
use bitflags::bitflags;
use glam::Vec2;

use crate::{
    entity::{
        direction::Direction,
        graph::{Graph, TraversalFlags},
    },
    error::{EntityError, GameResult},
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
    Wall,
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

/// A unique identifier for a node, represented by its index in the graph's storage.
pub type NodeId = usize;

/// Progress along an edge between two nodes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EdgeProgress {
    pub target_node: NodeId,
    /// Progress from 0.0 (at source node) to 1.0 (at target node)
    pub progress: f32,
}

/// Pure spatial position component - works for both static and dynamic entities.
#[derive(Component, Debug, Copy, Clone, PartialEq)]
pub struct Position {
    /// The current/primary node this entity is at or traveling from
    pub node: NodeId,
    /// If Some, entity is traveling between nodes. If None, entity is stationary at node.
    pub edge_progress: Option<EdgeProgress>,
}

/// Explicit movement state - only for entities that can move.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub enum MovementState {
    Stopped,
    Moving { direction: Direction },
}

/// Movement capability and parameters - only for entities that can move.
#[derive(Component, Debug, Clone, Copy)]
pub struct Movable {
    pub speed: f32,
    pub current_direction: Direction,
    pub requested_direction: Option<Direction>,
}

impl Position {
    /// Calculates the current pixel position in the game world.
    ///
    /// Converts the graph position to screen coordinates, accounting for
    /// the board offset and centering the sprite.
    ///
    /// # Errors
    ///
    /// Returns an `EntityError` if the node or edge is not found.
    pub fn get_pixel_pos(&self, graph: &Graph) -> GameResult<Vec2> {
        let pos = match &self.edge_progress {
            None => {
                // Entity is stationary at a node
                let node = graph.get_node(self.node).ok_or(EntityError::NodeNotFound(self.node))?;
                node.position
            }
            Some(edge_progress) => {
                // Entity is traveling between nodes
                let from_node = graph.get_node(self.node).ok_or(EntityError::NodeNotFound(self.node))?;
                let to_node = graph
                    .get_node(edge_progress.target_node)
                    .ok_or(EntityError::NodeNotFound(edge_progress.target_node))?;

                // For zero-distance edges (tunnels), progress >= 1.0 means we're at the target
                if edge_progress.progress >= 1.0 {
                    to_node.position
                } else {
                    // Interpolate position based on progress
                    from_node.position + (to_node.position - from_node.position) * edge_progress.progress
                }
            }
        };

        Ok(Vec2::new(
            pos.x + crate::constants::BOARD_PIXEL_OFFSET.x as f32,
            pos.y + crate::constants::BOARD_PIXEL_OFFSET.y as f32,
        ))
    }
}

impl Default for Position {
    fn default() -> Self {
        Position {
            node: 0,
            edge_progress: None,
        }
    }
}

#[allow(dead_code)]
impl Position {
    /// Returns `true` if the position is exactly at a node (not traveling).
    pub fn is_at_node(&self) -> bool {
        self.edge_progress.is_none()
    }

    /// Returns the `NodeId` of the current node (source of travel if moving).
    pub fn current_node(&self) -> NodeId {
        self.node
    }

    /// Returns the `NodeId` of the destination node, if currently traveling.
    pub fn target_node(&self) -> Option<NodeId> {
        self.edge_progress.as_ref().map(|ep| ep.target_node)
    }

    /// Returns `true` if the entity is traveling between nodes.
    pub fn is_moving(&self) -> bool {
        self.edge_progress.is_some()
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

#[derive(Component)]
pub struct Collider {
    pub size: f32,
    pub layer: CollisionLayer,
}

/// Marker components for collision filtering optimization
#[derive(Component)]
pub struct PacmanCollider;

#[derive(Component)]
pub struct GhostCollider;

#[derive(Component)]
pub struct ItemCollider;

#[derive(Component)]
pub struct Score(pub u32);

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
    pub score: Score,
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
