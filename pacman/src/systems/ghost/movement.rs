//! Ghost movement system with targeting, speed calculation, and special behaviors.

use super::{
    elroy::{elroy_speed, BlinkyMarker, Elroy},
    personality::{calculate_chase_target, TargetingContext},
    targeting::{choose_direction_at_intersection, RedZoneNodes},
    GhostModeController, GhostState, GhostType, ScatterChaseMode,
};
use crate::constants;
use crate::map::builder::Map;
use crate::platform::rng;
use crate::systems::common::{DeltaTime, Frozen};
use crate::systems::movement::{NodeId, Position, Velocity};
use crate::systems::player::PlayerControlled;
use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;

/// Component storing the ghost's current target node
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct GhostTarget(pub Option<NodeId>);

/// Nodes where ghosts slow down (tunnel segments).
#[derive(Resource, Debug)]
pub struct TunnelNodes(pub Vec<NodeId>);

impl TunnelNodes {
    pub fn from_map(map: &Map) -> Self {
        use crate::constants::MapTile;

        let nodes: Vec<NodeId> = map
            .iter_nodes()
            .filter_map(|(node_id, tile)| matches!(tile, MapTile::Tunnel).then_some(*node_id))
            .collect();

        Self(nodes)
    }

    pub fn contains(&self, node: NodeId) -> bool {
        self.0.contains(&node)
    }
}

/// Speed configuration based on level and state.
/// Inserted as a resource during init; update when the level changes.
#[derive(Resource, Debug, Clone)]
pub struct GhostSpeedConfig {
    pub normal: f32,
    pub tunnel: f32,
    pub frightened: f32,
}

impl GhostSpeedConfig {
    pub fn for_level(level: u8) -> Self {
        match level {
            1 => Self {
                normal: 0.75,
                tunnel: 0.40,
                frightened: 0.50,
            },
            2..=4 => Self {
                normal: 0.85,
                tunnel: 0.45,
                frightened: 0.55,
            },
            5..=20 => Self {
                normal: 0.95,
                tunnel: 0.50,
                frightened: 0.60,
            },
            _ => Self {
                normal: 0.95,
                tunnel: 0.50,
                frightened: 0.0,
            }, // No frightened after 20
        }
    }
}

/// System to update ghost targets based on mode and personality
pub fn ghost_targeting_system(
    map: Res<Map>,
    mode_controller: Res<GhostModeController>,
    pacman_query: Query<(&Position, &Velocity), With<PlayerControlled>>,
    mut ghost_query: Query<(&GhostType, &GhostState, &Position, &mut GhostTarget)>,
    blinky_query: Query<&Position, With<BlinkyMarker>>,
    elroy_query: Query<&Elroy, With<BlinkyMarker>>,
) {
    let Ok((pac_pos, pac_vel)) = pacman_query.single() else {
        return;
    };

    let pac_node = pac_pos.current_node();
    let pac_pixel_pos = pac_pos.get_pixel_position(&map.graph).unwrap_or_default();
    let pac_direction = pac_vel.direction;

    let blinky_pos = blinky_query
        .single()
        .ok()
        .and_then(|p| p.get_pixel_position(&map.graph).ok())
        .unwrap_or_default();

    for (ghost_type, state, position, mut target) in ghost_query.iter_mut() {
        // Only calculate targets for active ghosts (not in house or returning as eyes)
        if !matches!(state, GhostState::Active { .. }) {
            target.0 = None;
            continue;
        }

        let self_node = position.current_node();
        let self_pos = position.get_pixel_position(&map.graph).unwrap_or_default();

        let ctx = TargetingContext {
            pacman_node: pac_node,
            pacman_direction: pac_direction,
            pacman_position: pac_pixel_pos,
            blinky_position: blinky_pos,
            self_node,
            self_position: self_pos,
        };

        // Determine mode: frightened overrides global mode
        let effective_mode = if state.is_frightened() {
            // Frightened ghosts don't need targets (random movement)
            target.0 = None;
            continue;
        } else if let Ok(elroy) = elroy_query.single() {
            // Blinky in Elroy mode always chases even during scatter
            if *ghost_type == GhostType::Blinky && elroy.stage != super::elroy::ElroyStage::None {
                ScatterChaseMode::Chase
            } else {
                mode_controller.mode
            }
        } else {
            mode_controller.mode
        };

        let target_node = match effective_mode {
            ScatterChaseMode::Scatter => get_scatter_target(*ghost_type, &map),
            ScatterChaseMode::Chase => {
                let chase_target = calculate_chase_target(*ghost_type, &ctx, &map);
                // Special case for Clyde: if he returned self_node, use scatter instead
                if chase_target == self_node && *ghost_type == GhostType::Clyde {
                    get_scatter_target(GhostType::Clyde, &map)
                } else {
                    chase_target
                }
            }
        };

        target.0 = Some(target_node);
    }
}

/// Ghost movement configuration resources grouped as a SystemParam.
#[derive(SystemParam)]
pub struct GhostMovementConfig<'w> {
    pub mode_controller: Res<'w, GhostModeController>,
    pub speed_config: Res<'w, GhostSpeedConfig>,
    pub tunnel_nodes: Res<'w, TunnelNodes>,
    pub red_zones: Res<'w, RedZoneNodes>,
}

/// Main ghost movement system
pub fn ghost_movement_system(
    map: Res<Map>,
    delta_time: Res<DeltaTime>,
    config: GhostMovementConfig,
    elroy_query: Query<&Elroy, With<BlinkyMarker>>,
    mut ghost_query: Query<(&GhostType, &GhostState, &mut Position, &mut Velocity, &GhostTarget), Without<Frozen>>,
) {
    for (ghost_type, state, mut position, mut velocity, target) in ghost_query.iter_mut() {
        // Skip ghosts not active in the maze
        if !matches!(state, GhostState::Active { .. }) {
            continue;
        }

        // Calculate effective speed
        let elroy_mult = if *ghost_type == GhostType::Blinky {
            elroy_query
                .single()
                .map(|e| elroy_speed(e.stage, config.mode_controller.level))
                .unwrap_or(1.0)
        } else {
            1.0
        };

        let base_speed = calculate_speed(&position, state, &config.speed_config, &config.tunnel_nodes) * elroy_mult;

        velocity.speed = base_speed;

        let mut distance = velocity.speed * constants::TICKS_PER_SECOND * delta_time.seconds;

        loop {
            match *position {
                Position::Stopped { node } => {
                    // At intersection - choose direction based on target
                    let target_node = target.0.unwrap_or(node);
                    let is_frightened = state.is_frightened();

                    let mut rng = if is_frightened { Some(rng()) } else { None };

                    let new_direction = choose_direction_at_intersection(
                        &map,
                        &config.red_zones,
                        node,
                        target_node,
                        velocity.direction,
                        is_frightened,
                        rng.as_mut().map(|r| r as &mut dyn rand::RngCore),
                    );

                    if let Some(dir) = new_direction {
                        velocity.direction = dir;
                        let edge = map.graph.adjacency_list[node as usize]
                            .get(dir)
                            .expect("Direction should have valid edge");

                        *position = Position::Moving {
                            from: node,
                            to: edge.target,
                            remaining_distance: edge.distance,
                        };
                    } else {
                        break; // Stuck (shouldn't happen)
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

fn calculate_speed(position: &Position, state: &GhostState, config: &GhostSpeedConfig, tunnel_nodes: &TunnelNodes) -> f32 {
    let current_node = position.current_node();

    // Check tunnel slowdown
    if tunnel_nodes.contains(current_node) {
        return config.tunnel;
    }

    // Check frightened slowdown
    if state.is_frightened() {
        return config.frightened;
    }

    config.normal
}

fn get_scatter_target(ghost_type: GhostType, map: &Map) -> NodeId {
    match ghost_type {
        GhostType::Blinky => map.scatter_targets.blinky,
        GhostType::Pinky => map.scatter_targets.pinky,
        GhostType::Inky => map.scatter_targets.inky,
        GhostType::Clyde => map.scatter_targets.clyde,
    }
}
