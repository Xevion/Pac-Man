use crate::entity::direction::Direction;
use crate::entity::graph::Graph;
use crate::error::{EntityError, GameResult};
use bevy_ecs::component::Component;
use glam::Vec2;

/// A unique identifier for a node, represented by its index in the graph's storage.
pub type NodeId = usize;

/// A component that represents the speed and cardinal direction of an entity.
/// Speed is static, only applied when the entity has an edge to traverse.
/// Direction is dynamic, but is controlled externally.
#[derive(Component, Debug, Copy, Clone, PartialEq)]
pub struct Velocity {
    pub speed: f32,
    pub direction: Direction,
}

/// A component that represents a direction change that is only remembered for a period of time.
/// This is used to allow entities to change direction before they reach their current target node (which consumes their buffered direction).
#[derive(Component, Debug, Copy, Clone, PartialEq)]
pub enum BufferedDirection {
    None,
    Some { direction: Direction, remaining_time: f32 },
}

/// Pure spatial position component - works for both static and dynamic entities.
#[derive(Component, Debug, Copy, Clone, PartialEq)]
pub enum Position {
    Stopped {
        node: NodeId,
    },
    Moving {
        from: NodeId,
        to: NodeId,
        remaining_distance: f32,
    },
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
    pub fn get_pixel_position(&self, graph: &Graph) -> GameResult<Vec2> {
        let pos = match &self {
            Position::Stopped { node } => {
                // Entity is stationary at a node
                let node = graph.get_node(*node).ok_or(EntityError::NodeNotFound(*node))?;
                node.position
            }
            Position::Moving {
                from,
                to,
                remaining_distance,
            } => {
                // Entity is traveling between nodes
                let from_node = graph.get_node(*from).ok_or(EntityError::NodeNotFound(*from))?;
                let to_node = graph.get_node(*to).ok_or(EntityError::NodeNotFound(*to))?;
                let edge = graph
                    .find_edge(*from, *to)
                    .ok_or(EntityError::EdgeNotFound { from: *from, to: *to })?;

                // For zero-distance edges (tunnels), progress >= 1.0 means we're at the target
                if edge.distance == 0.0 {
                    to_node.position
                } else {
                    // Interpolate position based on progress
                    let progress = 1.0 - (*remaining_distance / edge.distance);
                    from_node.position.lerp(to_node.position, progress)
                }
            }
        };

        Ok(Vec2::new(
            pos.x + crate::constants::BOARD_PIXEL_OFFSET.x as f32,
            pos.y + crate::constants::BOARD_PIXEL_OFFSET.y as f32,
        ))
    }

    /// Moves the position by a given distance towards it's current target node.
    ///
    /// Returns the overflow distance, if any.
    pub fn tick(&mut self, distance: f32) -> Option<f32> {
        if distance <= 0.0 || self.is_at_node() {
            return None;
        }

        match self {
            Position::Moving {
                to, remaining_distance, ..
            } => {
                // If the remaining distance is less than or equal the distance, we'll reach the target
                if *remaining_distance <= distance {
                    let overflow: Option<f32> = if *remaining_distance != distance {
                        Some(distance - *remaining_distance)
                    } else {
                        None
                    };
                    *self = Position::Stopped { node: *to };

                    return overflow;
                }

                *remaining_distance -= distance;

                None
            }
            _ => unreachable!(),
        }
    }

    /// Returns `true` if the position is exactly at a node (not traveling).
    pub fn is_at_node(&self) -> bool {
        matches!(self, Position::Stopped { .. })
    }

    /// Returns the `NodeId` of the current node (source of travel if moving).
    pub fn current_node(&self) -> NodeId {
        match self {
            Position::Stopped { node } => *node,
            Position::Moving { from, .. } => *from,
        }
    }

    /// Returns the `NodeId` of the destination node, if currently traveling.
    pub fn target_node(&self) -> Option<NodeId> {
        match self {
            Position::Stopped { .. } => None,
            Position::Moving { to, .. } => Some(*to),
        }
    }

    /// Returns `true` if the entity is traveling between nodes.
    pub fn is_moving(&self) -> bool {
        matches!(self, Position::Moving { .. })
    }
}

// pub fn movement_system(
//     map: Res<Map>,
//     delta_time: Res<DeltaTime>,
//     mut entities: Query<(&mut Position, &mut Movable, &EntityType)>,
//     mut errors: EventWriter<GameError>,
// ) {
//     for (mut position, mut movable, entity_type) in entities.iter_mut() {
//         let distance = movable.speed * 60.0 * delta_time.0;

//         match *position {
//             Position::Stopped { .. } => {
//                 // Check if we have a requested direction to start moving
//                 if let Some(requested_direction) = movable.requested_direction {
//                     if let Some(edge) = map.graph.find_edge_in_direction(position.current_node(), requested_direction) {
//                         if can_traverse(*entity_type, edge) {
//                             // Start moving in the requested direction
//                             let progress = if edge.distance > 0.0 {
//                                 distance / edge.distance
//                             } else {
//                                 // Zero-distance edge (tunnels) - immediately teleport
//                                 tracing::debug!(
//                                     "Entity entering tunnel from node {} to node {}",
//                                     position.current_node(),
//                                     edge.target
//                                 );
//                                 1.0
//                             };

//                             *position = Position::Moving {
//                                 from: position.current_node(),
//                                 to: edge.target,
//                                 remaining_distance: progress,
//                             };
//                             movable.current_direction = requested_direction;
//                             movable.requested_direction = None;
//                         }
//                     } else {
//                         errors.write(
//                             EntityError::InvalidMovement(format!(
//                                 "No edge found in direction {:?} from node {}",
//                                 requested_direction,
//                                 position.current_node()
//                             ))
//                             .into(),
//                         );
//                     }
//                 }
//             }
//             Position::Moving {
//                 from,
//                 to,
//                 remaining_distance,
//             } => {
//                 // Continue moving or handle node transitions
//                 let current_node = *from;
//                 if let Some(edge) = map.graph.find_edge(current_node, *to) {
//                     // Extract target node before mutable operations
//                     let target_node = *to;

//                     // Get the current edge for distance calculation
//                     let edge = map.graph.find_edge(current_node, target_node);

//                     if let Some(edge) = edge {
//                         // Update progress along the edge
//                         if edge.distance > 0.0 {
//                             *remaining_distance += distance / edge.distance;
//                         } else {
//                             // Zero-distance edge (tunnels) - immediately complete
//                             *remaining_distance = 1.0;
//                         }

//                         if *remaining_distance >= 1.0 {
//                             // Reached the target node
//                             let overflow = if edge.distance > 0.0 {
//                                 (*remaining_distance - 1.0) * edge.distance
//                             } else {
//                                 // Zero-distance edge - use remaining distance for overflow
//                                 distance
//                             };
//                             *position = Position::Stopped { node: target_node };

//                             let mut continued_moving = false;

//                             // Try to use requested direction first
//                             if let Some(requested_direction) = movable.requested_direction {
//                                 if let Some(next_edge) = map.graph.find_edge_in_direction(position.node, requested_direction) {
//                                     if can_traverse(*entity_type, next_edge) {
//                                         let next_progress = if next_edge.distance > 0.0 {
//                                             overflow / next_edge.distance
//                                         } else {
//                                             // Zero-distance edge - immediately complete
//                                             1.0
//                                         };

//                                         *position = Position::Moving {
//                                             from: position.current_node(),
//                                             to: next_edge.target,
//                                             remaining_distance: next_progress,
//                                         };
//                                         movable.current_direction = requested_direction;
//                                         movable.requested_direction = None;
//                                         continued_moving = true;
//                                     }
//                                 }
//                             }

//                             // If no requested direction or it failed, try to continue in current direction
//                             if !continued_moving {
//                                 if let Some(next_edge) = map.graph.find_edge_in_direction(position.node, direction) {
//                                     if can_traverse(*entity_type, next_edge) {
//                                         let next_progress = if next_edge.distance > 0.0 {
//                                             overflow / next_edge.distance
//                                         } else {
//                                             // Zero-distance edge - immediately complete
//                                             1.0
//                                         };

//                                         *position = Position::Moving {
//                                             from: position.current_node(),
//                                             to: next_edge.target,
//                                             remaining_distance: next_progress,
//                                         };
//                                         // Keep current direction and movement state
//                                         continued_moving = true;
//                                     }
//                                 }
//                             }

//                             // If we couldn't continue moving, stop
//                             if !continued_moving {
//                                 *movement_state = MovementState::Stopped;
//                                 movable.requested_direction = None;
//                             }
//                         }
//                     } else {
//                         // Edge not found - this is an inconsistent state
//                         errors.write(
//                             EntityError::InvalidMovement(format!(
//                                 "Inconsistent state: Moving on non-existent edge from {} to {}",
//                                 current_node, target_node
//                             ))
//                             .into(),
//                         );
//                         *movement_state = MovementState::Stopped;
//                         position.edge_progress = None;
//                     }
//                 } else {
//                     // Movement state says moving but no edge progress - this shouldn't happen
//                     errors.write(EntityError::InvalidMovement("Entity in Moving state but no edge progress".to_string()).into());
//                     *movement_state = MovementState::Stopped;
//                 }
//             }
//         }
//     }
// }
