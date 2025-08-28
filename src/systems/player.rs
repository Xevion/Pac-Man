use bevy_ecs::{
    component::Component,
    event::{EventReader, EventWriter},
    query::With,
    system::{Query, Res, ResMut},
};

use crate::{
    error::GameError,
    events::{GameCommand, GameEvent},
    map::{builder::Map, graph::Edge},
    systems::{
        components::{DeltaTime, EntityType, GlobalState, MovementModifiers, PlayerControlled},
        debug::DebugState,
        movement::{BufferedDirection, Position, Velocity},
        AudioState,
    },
};

/// Lifecycle state for the player entity.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerLifecycle {
    Spawning,
    Alive,
    Dying,
    Respawning,
}

impl PlayerLifecycle {
    /// Returns true when gameplay input and movement should be active
    pub fn is_interactive(self) -> bool {
        matches!(self, PlayerLifecycle::Alive)
    }
}

impl Default for PlayerLifecycle {
    fn default() -> Self {
        PlayerLifecycle::Spawning
    }
}

/// Whether player input should be processed.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlState {
    InputEnabled,
    InputLocked,
}

impl Default for ControlState {
    fn default() -> Self {
        Self::InputLocked
    }
}

/// Combat-related state for Pac-Man. Tick-based energizer logic.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatState {
    Normal,
    Energized {
        /// Remaining energizer duration in ticks (frames)
        remaining_ticks: u32,
        /// Ticks until flashing begins (counts down to 0, then flashing is active)
        flash_countdown_ticks: u32,
    },
}

impl Default for CombatState {
    fn default() -> Self {
        CombatState::Normal
    }
}

impl CombatState {
    pub fn is_energized(&self) -> bool {
        matches!(self, CombatState::Energized { .. })
    }

    pub fn is_flashing(&self) -> bool {
        matches!(self, CombatState::Energized { flash_countdown_ticks, .. } if *flash_countdown_ticks == 0)
    }

    pub fn deactivate_energizer(&mut self) {
        *self = CombatState::Normal;
    }

    /// Activate energizer using tick-based durations.
    pub fn activate_energizer_ticks(&mut self, total_ticks: u32, flash_lead_ticks: u32) {
        let flash_countdown_ticks = total_ticks.saturating_sub(flash_lead_ticks);
        *self = CombatState::Energized {
            remaining_ticks: total_ticks,
            flash_countdown_ticks,
        };
    }

    /// Advance one frame. When ticks reach zero, returns to Normal.
    pub fn tick_frame(&mut self) {
        if let CombatState::Energized {
            remaining_ticks,
            flash_countdown_ticks,
        } = self
        {
            if *remaining_ticks > 0 {
                *remaining_ticks -= 1;
                if *flash_countdown_ticks > 0 {
                    *flash_countdown_ticks -= 1;
                }
            }
            if *remaining_ticks == 0 {
                *self = CombatState::Normal;
            }
        }
    }
}

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
    mut players: Query<(&PlayerLifecycle, &ControlState, &mut BufferedDirection), With<PlayerControlled>>,
    mut errors: EventWriter<GameError>,
) {
    // Get the player's movable component (ensuring there is only one player)
    let (lifecycle, control, mut buffered_direction) = match players.single_mut() {
        Ok(tuple) => tuple,
        Err(e) => {
            errors.write(GameError::InvalidState(format!(
                "No/multiple entities queried for player system: {}",
                e
            )));
            return;
        }
    };

    // If the player is not interactive or input is locked, ignore movement commands
    let allow_input = lifecycle.is_interactive() && matches!(control, ControlState::InputEnabled);

    // Handle events
    for event in events.read() {
        if let GameEvent::Command(command) = event {
            match command {
                GameCommand::MovePlayer(direction) => {
                    if allow_input {
                        *buffered_direction = BufferedDirection::Some {
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

pub fn can_traverse(entity_type: EntityType, edge: Edge) -> bool {
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
    mut entities: Query<
        (
            &PlayerLifecycle,
            &ControlState,
            &MovementModifiers,
            &mut Position,
            &mut Velocity,
            &mut BufferedDirection,
        ),
        With<PlayerControlled>,
    >,
    // mut errors: EventWriter<GameError>,
) {
    for (lifecycle, control, modifiers, mut position, mut velocity, mut buffered_direction) in entities.iter_mut() {
        if !lifecycle.is_interactive() || !matches!(control, ControlState::InputEnabled) {
            continue;
        }

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

        let mut distance = velocity.speed * modifiers.speed_multiplier * 60.0 * delta_time.0;

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

/// Applies tunnel slowdown based on the current node tile
pub fn player_tunnel_slowdown_system(map: Res<Map>, mut q: Query<(&Position, &mut MovementModifiers), With<PlayerControlled>>) {
    if let Ok((position, mut modifiers)) = q.single_mut() {
        let node = position.current_node();
        let in_tunnel = map
            .tile_at_node(node)
            .map(|t| t == crate::constants::MapTile::Tunnel)
            .unwrap_or(false);
        modifiers.tunnel_slowdown_active = in_tunnel;
        modifiers.speed_multiplier = if in_tunnel { 0.6 } else { 1.0 };
    }
}
