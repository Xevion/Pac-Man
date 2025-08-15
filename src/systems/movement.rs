use crate::entity::graph::{Edge, EdgePermissions, Graph};
use crate::error::{EntityError, GameError};
use crate::map::builder::Map;
use crate::systems::components::{DeltaTime, EntityType, Position, Velocity};
use bevy_ecs::event::EventWriter;
use bevy_ecs::system::{Query, Res};

fn can_traverse(entity_type: EntityType, edge: Edge) -> bool {
    match entity_type {
        EntityType::Player => matches!(edge.permissions, EdgePermissions::All),
        EntityType::Ghost => matches!(edge.permissions, EdgePermissions::All | EdgePermissions::GhostsOnly),
        _ => matches!(edge.permissions, EdgePermissions::All),
    }
}

pub fn movement_system(
    map: Res<Map>,
    delta_time: Res<DeltaTime>,
    mut entities: Query<(&mut Velocity, &mut Position, &EntityType)>,
    mut errors: EventWriter<GameError>,
) {
    for (mut velocity, mut position, entity_type) in entities.iter_mut() {
        let distance = velocity.speed * 60.0 * delta_time.0;

        // Decrement the remaining frames for the next direction
        if let Some((direction, remaining)) = velocity.next_direction {
            if remaining > 0 {
                velocity.next_direction = Some((direction, remaining - 1));
            } else {
                velocity.next_direction = None;
            }
        }

        match *position {
            Position::AtNode(node_id) => {
                // We're not moving, but a buffered direction is available.
                if let Some((next_direction, _)) = velocity.next_direction {
                    if let Some(edge) = map.graph.find_edge_in_direction(node_id, next_direction) {
                        if can_traverse(*entity_type, edge) {
                            // Start moving in that direction
                            *position = Position::BetweenNodes {
                                from: node_id,
                                to: edge.target,
                                traversed: distance,
                            };
                            velocity.direction = next_direction;
                            velocity.next_direction = None;
                        }
                    } else {
                        errors.write(
                            EntityError::InvalidMovement(format!(
                                "No edge found in direction {:?} from node {}",
                                next_direction, node_id
                            ))
                            .into(),
                        );
                    }
                }
            }
            Position::BetweenNodes { from, to, traversed } => {
                // There is no point in any of the next logic if we don't travel at all
                if distance <= 0.0 {
                    return;
                }

                let edge = map
                    .graph
                    .find_edge(from, to)
                    .ok_or_else(|| {
                        errors.write(
                            EntityError::InvalidMovement(format!(
                                "Inconsistent state: Traverser is on a non-existent edge from {} to {}.",
                                from, to
                            ))
                            .into(),
                        );
                        return;
                    })
                    .unwrap();

                let new_traversed = traversed + distance;

                if new_traversed < edge.distance {
                    // Still on the same edge, just update the distance.
                    *position = Position::BetweenNodes {
                        from,
                        to,
                        traversed: new_traversed,
                    };
                } else {
                    let overflow = new_traversed - edge.distance;
                    let mut moved = false;

                    // If we buffered a direction, try to find an edge in that direction
                    if let Some((next_dir, _)) = velocity.next_direction {
                        if let Some(edge) = map.graph.find_edge_in_direction(to, next_dir) {
                            if can_traverse(*entity_type, edge) {
                                *position = Position::BetweenNodes {
                                    from: to,
                                    to: edge.target,
                                    traversed: overflow,
                                };

                                velocity.direction = next_dir; // Remember our new direction
                                velocity.next_direction = None; // Consume the buffered direction
                                moved = true;
                            }
                        }
                    }

                    // If we didn't move, try to continue in the current direction
                    if !moved {
                        if let Some(edge) = map.graph.find_edge_in_direction(to, velocity.direction) {
                            if can_traverse(*entity_type, edge) {
                                *position = Position::BetweenNodes {
                                    from: to,
                                    to: edge.target,
                                    traversed: overflow,
                                };
                            } else {
                                *position = Position::AtNode(to);
                                velocity.next_direction = None;
                            }
                        } else {
                            *position = Position::AtNode(to);
                            velocity.next_direction = None;
                        }
                    }
                }
            }
        }
    }
}
