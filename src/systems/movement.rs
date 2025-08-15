use crate::entity::graph::Edge;
use crate::error::{EntityError, GameError};
use crate::map::builder::Map;
use crate::systems::components::{DeltaTime, EdgeProgress, EntityType, Movable, MovementState, Position};
use bevy_ecs::event::EventWriter;
use bevy_ecs::system::{Query, Res};

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
