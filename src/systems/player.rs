use bevy_ecs::{
    component::Component,
    event::EventReader,
    query::{With, Without},
    system::{Query, Res, ResMut, Single},
};
use tracing::trace;

use crate::{
    events::{GameCommand, GameEvent},
    map::{builder::Map, graph::Edge},
    systems::{
        components::{DeltaTime, EntityType, Frozen, GlobalState, MovementModifiers},
        debug::DebugState,
        movement::{BufferedDirection, Position, Velocity},
        AudioState,
    },
};

/// A tag component for entities that are controlled by the player.
#[derive(Default, Component)]
pub struct PlayerControlled;

pub fn can_traverse(entity_type: EntityType, edge: Edge) -> bool {
    let entity_flags = entity_type.traversal_flags();
    edge.traversal_flags.contains(entity_flags)
}

/// Processes player input commands and updates game state accordingly.
///
/// Handles keyboard-driven commands like movement direction changes, debug mode
/// toggling, audio muting, and game exit requests. Movement commands are buffered
/// to allow direction changes before reaching intersections, improving gameplay
/// responsiveness. Non-movement commands immediately modify global game state.
#[allow(clippy::type_complexity)]
pub fn player_control_system(
    mut events: EventReader<GameEvent>,
    mut state: ResMut<GlobalState>,
    mut debug_state: ResMut<DebugState>,
    mut audio_state: ResMut<AudioState>,
    mut player: Option<Single<&mut BufferedDirection, (With<PlayerControlled>, Without<Frozen>)>>,
) {
    // Handle events
    for event in events.read() {
        if let GameEvent::Command(command) = event {
            match command {
                GameCommand::MovePlayer(direction) => {
                    // Only handle movement if there's an unfrozen player
                    if let Some(player_single) = player.as_mut() {
                        trace!(direction = ?*direction, "Player direction buffered for movement");
                        ***player_single = BufferedDirection::Some {
                            direction: *direction,
                            remaining_time: 0.25,
                        };
                    }
                }
                GameCommand::Exit => {
                    state.exit = true;
                }
                GameCommand::ToggleDebug => {
                    debug_state.enabled = !debug_state.enabled;
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

/// Executes frame-by-frame movement for Pac-Man.
///
/// Handles movement logic including buffered direction changes, edge traversal validation, and continuous movement between nodes.
/// When stopped, prioritizes buffered directions for responsive controls, falling back to current direction.
/// Supports movement chaining within a single frame when traveling at high speeds.
#[allow(clippy::type_complexity)]
pub fn player_movement_system(
    map: Res<Map>,
    delta_time: Res<DeltaTime>,
    mut entities: Query<
        (&MovementModifiers, &mut Position, &mut Velocity, &mut BufferedDirection),
        (With<PlayerControlled>, Without<Frozen>),
    >,
    mut last_stopped_node: bevy_ecs::system::Local<Option<crate::systems::movement::NodeId>>,
) {
    for (modifiers, mut position, mut velocity, mut buffered_direction) in entities.iter_mut() {
        // Decrement the buffered direction remaining time
        if let BufferedDirection::Some {
            direction,
            remaining_time,
        } = *buffered_direction
        {
            if remaining_time <= 0.0 {
                trace!("Buffered direction expired");
                *buffered_direction = BufferedDirection::None;
            } else {
                *buffered_direction = BufferedDirection::Some {
                    direction,
                    remaining_time: remaining_time - delta_time.seconds,
                };
            }
        }

        let mut distance = velocity.speed * modifiers.speed_multiplier * 60.0 * delta_time.seconds;

        loop {
            match *position {
                Position::Stopped { .. } => {
                    // If there is a buffered direction, travel it's edge first if available.
                    if let BufferedDirection::Some { direction, .. } = *buffered_direction {
                        // If there's no edge in that direction, ignore the buffered direction.
                        if let Some(edge) = map.graph.find_edge_in_direction(position.current_node(), direction) {
                            // If there is an edge in that direction (and it's traversable), start moving towards it and consume the buffered direction.
                            if can_traverse(EntityType::Player, edge) {
                                trace!(from = position.current_node(), to = edge.target, direction = ?direction, "Player started moving using buffered direction");
                                *last_stopped_node = None; // Reset stopped state when starting to move
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
                            trace!(from = position.current_node(), to = edge.target, direction = ?velocity.direction, "Player continued in current direction");
                            *last_stopped_node = None; // Reset stopped state when starting to move
                            velocity.direction = edge.direction;
                            *position = Position::Moving {
                                from: position.current_node(),
                                to: edge.target,
                                remaining_distance: edge.distance,
                            };
                        }
                    } else {
                        // No edge in our current direction either, erase the buffered direction and stop.
                        let current_node = position.current_node();
                        if *last_stopped_node != Some(current_node) {
                            trace!(node = current_node, direction = ?velocity.direction, "Player stopped - no valid edge in current direction");
                            *last_stopped_node = Some(current_node);
                        }
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

/// Applies tunnel slowdown based on the current node tile
pub fn player_tunnel_slowdown_system(map: Res<Map>, player: Single<(&Position, &mut MovementModifiers), With<PlayerControlled>>) {
    let (position, mut modifiers) = player.into_inner();
    let node = position.current_node();
    let in_tunnel = map
        .tile_at_node(node)
        .map(|t| t == crate::constants::MapTile::Tunnel)
        .unwrap_or(false);

    if modifiers.tunnel_slowdown_active != in_tunnel {
        trace!(
            node,
            in_tunnel,
            speed_multiplier = if in_tunnel { 0.6 } else { 1.0 },
            "Player tunnel slowdown state changed"
        );
    }

    modifiers.tunnel_slowdown_active = in_tunnel;
    modifiers.speed_multiplier = if in_tunnel { 0.6 } else { 1.0 };
}
