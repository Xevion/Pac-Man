use crate::entity::graph::Graph;
use crate::entity::{direction::Direction, graph::Edge};
use crate::error::{EntityError, GameError, GameResult};
use crate::map::builder::Map;
use crate::systems::components::{DeltaTime, EntityType};
use bevy_ecs::component::Component;
use bevy_ecs::event::EventWriter;
use bevy_ecs::system::{Query, Res};
use glam::Vec2;

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
#[derive(Component, Debug, Copy, Clone, PartialEq, Default)]
pub struct Position {
    /// The current/primary node this entity is at or traveling from
    pub node: NodeId,
    /// If Some, entity is traveling between nodes. If None, entity is stationary at node.
    pub edge_progress: Option<EdgeProgress>,
}

/// Explicit movement state - only for entities that can move.
#[derive(Component, Debug, Clone, Copy, PartialEq, Default)]
pub enum MovementState {
    #[default]
    Stopped,
    Moving {
        direction: Direction,
    },
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

fn can_traverse(entity_type: EntityType, edge: Edge) -> bool {
    let entity_flags = entity_type.traversal_flags();
    edge.traversal_flags.contains(entity_flags)
}

pub fn movement_system(
    map: Res<Map>,
    delta_time: Res<DeltaTime>,
    mut entities: Query<(&mut MovementState, &mut Movable, &mut Position, &EntityType)>,
    mut errors: EventWriter<GameError>,
) {
    for (mut movement_state, mut movable, mut position, entity_type) in entities.iter_mut() {
        let distance = movable.speed * 60.0 * delta_time.0;

        match *movement_state {
            MovementState::Stopped => {
                // Check if we have a requested direction to start moving
                if let Some(requested_direction) = movable.requested_direction {
                    if let Some(edge) = map.graph.find_edge_in_direction(position.node, requested_direction) {
                        if can_traverse(*entity_type, edge) {
                            // Start moving in the requested direction
                            let progress = if edge.distance > 0.0 {
                                distance / edge.distance
                            } else {
                                // Zero-distance edge (tunnels) - immediately teleport
                                tracing::debug!("Entity entering tunnel from node {} to node {}", position.node, edge.target);
                                1.0
                            };

                            position.edge_progress = Some(EdgeProgress {
                                target_node: edge.target,
                                progress,
                            });
                            movable.current_direction = requested_direction;
                            movable.requested_direction = None;
                            *movement_state = MovementState::Moving {
                                direction: requested_direction,
                            };
                        }
                    } else {
                        errors.write(
                            EntityError::InvalidMovement(format!(
                                "No edge found in direction {:?} from node {}",
                                requested_direction, position.node
                            ))
                            .into(),
                        );
                    }
                }
            }
            MovementState::Moving { direction } => {
                // Continue moving or handle node transitions
                let current_node = position.node;
                if let Some(edge_progress) = &mut position.edge_progress {
                    // Extract target node before mutable operations
                    let target_node = edge_progress.target_node;

                    // Get the current edge for distance calculation
                    let edge = map.graph.find_edge(current_node, target_node);

                    if let Some(edge) = edge {
                        // Update progress along the edge
                        if edge.distance > 0.0 {
                            edge_progress.progress += distance / edge.distance;
                        } else {
                            // Zero-distance edge (tunnels) - immediately complete
                            edge_progress.progress = 1.0;
                        }

                        if edge_progress.progress >= 1.0 {
                            // Reached the target node
                            let overflow = if edge.distance > 0.0 {
                                (edge_progress.progress - 1.0) * edge.distance
                            } else {
                                // Zero-distance edge - use remaining distance for overflow
                                distance
                            };
                            position.node = target_node;
                            position.edge_progress = None;

                            let mut continued_moving = false;

                            // Try to use requested direction first
                            if let Some(requested_direction) = movable.requested_direction {
                                if let Some(next_edge) = map.graph.find_edge_in_direction(position.node, requested_direction) {
                                    if can_traverse(*entity_type, next_edge) {
                                        let next_progress = if next_edge.distance > 0.0 {
                                            overflow / next_edge.distance
                                        } else {
                                            // Zero-distance edge - immediately complete
                                            1.0
                                        };

                                        position.edge_progress = Some(EdgeProgress {
                                            target_node: next_edge.target,
                                            progress: next_progress,
                                        });
                                        movable.current_direction = requested_direction;
                                        movable.requested_direction = None;
                                        *movement_state = MovementState::Moving {
                                            direction: requested_direction,
                                        };
                                        continued_moving = true;
                                    }
                                }
                            }

                            // If no requested direction or it failed, try to continue in current direction
                            if !continued_moving {
                                if let Some(next_edge) = map.graph.find_edge_in_direction(position.node, direction) {
                                    if can_traverse(*entity_type, next_edge) {
                                        let next_progress = if next_edge.distance > 0.0 {
                                            overflow / next_edge.distance
                                        } else {
                                            // Zero-distance edge - immediately complete
                                            1.0
                                        };

                                        position.edge_progress = Some(EdgeProgress {
                                            target_node: next_edge.target,
                                            progress: next_progress,
                                        });
                                        // Keep current direction and movement state
                                        continued_moving = true;
                                    }
                                }
                            }

                            // If we couldn't continue moving, stop
                            if !continued_moving {
                                *movement_state = MovementState::Stopped;
                                movable.requested_direction = None;
                            }
                        }
                    } else {
                        // Edge not found - this is an inconsistent state
                        errors.write(
                            EntityError::InvalidMovement(format!(
                                "Inconsistent state: Moving on non-existent edge from {} to {}",
                                current_node, target_node
                            ))
                            .into(),
                        );
                        *movement_state = MovementState::Stopped;
                        position.edge_progress = None;
                    }
                } else {
                    // Movement state says moving but no edge progress - this shouldn't happen
                    errors.write(EntityError::InvalidMovement("Entity in Moving state but no edge progress".to_string()).into());
                    *movement_state = MovementState::Stopped;
                }
            }
        }
    }
}
