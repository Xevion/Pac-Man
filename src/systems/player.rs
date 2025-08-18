use bevy_ecs::{
    event::{EventReader, EventWriter},
    prelude::ResMut,
    query::With,
    system::{Query, Res},
};

use crate::{
    error::GameError,
    events::{GameCommand, GameEvent},
    map::builder::Map,
    map::graph::Edge,
    systems::{
        components::{AudioState, DeltaTime, EntityType, GlobalState, PlayerControlled},
        debug::DebugState,
        movement::{BufferedDirection, Position, Velocity},
    },
};

/// Processes player input commands and updates game state accordingly.
///
/// Handles keyboard-driven commands like movement direction changes, debug mode
/// toggling, audio muting, and game exit requests. Movement commands are buffered
/// to allow direction changes before reaching intersections, improving gameplay
/// responsiveness. Non-movement commands immediately modify global game state.
pub fn player_control_system(
    mut events: EventReader<GameEvent>,
    mut state: ResMut<GlobalState>,
    mut debug_state: ResMut<DebugState>,
    mut audio_state: ResMut<AudioState>,
    mut players: Query<&mut BufferedDirection, With<PlayerControlled>>,
    mut errors: EventWriter<GameError>,
) {
    // Get the player's movable component (ensuring there is only one player)
    let mut buffered_direction = match players.single_mut() {
        Ok(buffered_direction) => buffered_direction,
        Err(e) => {
            errors.write(GameError::InvalidState(format!(
                "No/multiple entities queried for player system: {}",
                e
            )));
            return;
        }
    };

    // Handle events
    for event in events.read() {
        if let GameEvent::Command(command) = event {
            match command {
                GameCommand::MovePlayer(direction) => {
                    *buffered_direction = BufferedDirection::Some {
                        direction: *direction,
                        remaining_time: 0.25,
                    };
                }
                GameCommand::Exit => {
                    state.exit = true;
                }
                GameCommand::ToggleDebug => {
                    *debug_state = debug_state.next();
                }
                GameCommand::MuteAudio => {
                    audio_state.muted = !audio_state.muted;
                    tracing::info!("Audio {}", if audio_state.muted { "muted" } else { "unmuted" });
                }
                _ => {}
            }
        }
    }
}

fn can_traverse(entity_type: EntityType, edge: Edge) -> bool {
    let entity_flags = entity_type.traversal_flags();
    edge.traversal_flags.contains(entity_flags)
}

/// Executes frame-by-frame movement for Pac-Man.
///
/// Handles movement logic including buffered direction changes, edge traversal validation, and continuous movement between nodes.
/// When stopped, prioritizes buffered directions for responsive controls, falling back to current direction.
/// Supports movement chaining within a single frame when traveling at high speeds.
pub fn player_movement_system(
    map: Res<Map>,
    delta_time: Res<DeltaTime>,
    mut entities: Query<(&mut Position, &mut Velocity, &mut BufferedDirection), With<PlayerControlled>>,
    // mut errors: EventWriter<GameError>,
) {
    for (mut position, mut velocity, mut buffered_direction) in entities.iter_mut() {
        // Decrement the buffered direction remaining time
        if let BufferedDirection::Some {
            direction,
            remaining_time,
        } = *buffered_direction
        {
            if remaining_time <= 0.0 {
                *buffered_direction = BufferedDirection::None;
            } else {
                *buffered_direction = BufferedDirection::Some {
                    direction,
                    remaining_time: remaining_time - delta_time.0,
                };
            }
        }

        let mut distance = velocity.speed * 60.0 * delta_time.0;

        loop {
            match *position {
                Position::Stopped { .. } => {
                    // If there is a buffered direction, travel it's edge first if available.
                    if let BufferedDirection::Some { direction, .. } = *buffered_direction {
                        // If there's no edge in that direction, ignore the buffered direction.
                        if let Some(edge) = map.graph.find_edge_in_direction(position.current_node(), direction) {
                            // If there is an edge in that direction (and it's traversable), start moving towards it and consume the buffered direction.
                            if can_traverse(EntityType::Player, edge) {
                                velocity.direction = edge.direction;
                                *position = Position::Moving {
                                    from: position.current_node(),
                                    to: edge.target,
                                    remaining_distance: edge.distance,
                                };
                                *buffered_direction = BufferedDirection::None;
                            }
                        }
                    }

                    // If there is no buffered direction (or it's not yet valid), continue in the current direction.
                    if let Some(edge) = map.graph.find_edge_in_direction(position.current_node(), velocity.direction) {
                        if can_traverse(EntityType::Player, edge) {
                            velocity.direction = edge.direction;
                            *position = Position::Moving {
                                from: position.current_node(),
                                to: edge.target,
                                remaining_distance: edge.distance,
                            };
                        }
                    } else {
                        // No edge in our current direction either, erase the buffered direction and stop.
                        *buffered_direction = BufferedDirection::None;
                        break;
                    }
                }
                Position::Moving { .. } => {
                    if let Some(overflow) = position.tick(distance) {
                        distance = overflow;
                    } else {
                        break;
                    }
                }
            }
        }
    }
}
