use bevy_ecs::{
    event::{EventReader, EventWriter},
    query::With,
    system::{Query, Res, ResMut},
};

use crate::{
    ecs::{DeltaTime, GlobalState, PlayerControlled, Position, Velocity},
    error::{EntityError, GameError},
    game::events::GameEvent,
    input::commands::GameCommand,
    map::builder::Map,
};

pub fn movement_system(
    map: Res<Map>,
    delta_time: Res<DeltaTime>,
    mut entities: Query<(&PlayerControlled, &mut Velocity, &mut Position)>,
    mut errors: EventWriter<GameError>,
) {
    for (player, mut velocity, mut position) in entities.iter_mut() {
        let distance = velocity.speed.unwrap_or(0.0) * delta_time.0;

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
                        // if edge.permissions.can_traverse(edge) {
                        //     // Start moving in that direction
                        *position = Position::BetweenNodes {
                            from: node_id,
                            to: edge.target,
                            traversed: distance,
                        };
                        velocity.direction = next_direction;
                        // } else {
                        //     return Err(crate::error::GameError::Entity(crate::error::EntityError::InvalidMovement(
                        //         format!(
                        //             "Cannot traverse edge from {} to {} in direction {:?}",
                        //             node_id, edge.target, next_direction
                        //         ),
                        //     )));
                        // }
                    } else {
                        errors.write(
                            EntityError::InvalidMovement(format!(
                                "No edge found in direction {:?} from node {}",
                                next_direction, node_id
                            ))
                            .into(),
                        );
                    }

                    velocity.next_direction = None; // Consume the buffered direction regardless of whether we started moving with it
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
                            // if edge.permissions.can_traverse(edge) {
                            //     *position = Position::BetweenNodes {
                            //         from: to,
                            //         to: edge.target,
                            //         traversed: overflow,
                            //     };

                            velocity.direction = next_dir; // Remember our new direction
                            velocity.next_direction = None; // Consume the buffered direction
                            moved = true;
                            // }
                        }
                    }

                    // If we didn't move, try to continue in the current direction
                    if !moved {
                        if let Some(edge) = map.graph.find_edge_in_direction(to, velocity.direction) {
                            // if edge.permissions.can_traverse(edge) {
                            *position = Position::BetweenNodes {
                                from: to,
                                to: edge.target,
                                traversed: overflow,
                            };
                            // } else {
                            //     *position = Position::AtNode(to);
                            //     velocity.next_direction = None;
                            // }
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

// Handles
pub fn interact_system(
    mut events: EventReader<GameEvent>,
    mut state: ResMut<GlobalState>,
    mut players: Query<(&PlayerControlled, &mut Velocity)>,
    mut errors: EventWriter<GameError>,
) {
    // Get the player's velocity (handling to ensure there is only one player)
    let mut velocity = match players.single_mut() {
        Ok((_, velocity)) => velocity,
        Err(e) => {
            errors.write(GameError::InvalidState(format!("Player not found: {}", e)).into());
            return;
        }
    };

    // Handle events
    for event in events.read() {
        match event {
            GameEvent::Command(command) => match command {
                GameCommand::MovePlayer(direction) => {
                    velocity.direction = *direction;
                }
                GameCommand::Exit => {
                    state.exit = true;
                }
                _ => {}
            },
        }
    }
}
