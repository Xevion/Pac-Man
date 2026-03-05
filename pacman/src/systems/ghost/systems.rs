//! Ghost-related ECS systems.

use super::{
    elroy::{elroy_thresholds, BlinkyMarker, Elroy, ElroyStage},
    GhostAnimationState, GhostAnimations, GhostHouseController, GhostModeController, GhostState, GhostType, LastAnimationState,
};
use crate::constants;
use crate::map::builder::Map;
use crate::map::direction::Direction;
use crate::map::graph::{Edge, TraversalFlags};
use crate::systems::animation::{DirectionalAnimation, LinearAnimation, Looping};
use crate::systems::common::{DeltaTime, Frozen};
use crate::systems::item::PelletCount;
use crate::systems::movement::{NodeId, Position, Velocity};
use bevy_ecs::prelude::*;
use tracing::{debug, trace};

/// System to tick the ghost mode controller and trigger reversals
pub fn ghost_mode_tick_system(
    mut mode_controller: ResMut<GhostModeController>,
    mut ghost_query: Query<(&GhostState, &mut crate::systems::movement::Velocity)>,
) {
    // Check if any ghosts are frightened
    let any_frightened = ghost_query.iter().any(|(state, _)| state.is_frightened());

    if any_frightened && !mode_controller.paused {
        mode_controller.pause();
    } else if !any_frightened && mode_controller.paused {
        mode_controller.resume();
    }

    // Tick and check for mode change
    let mode_changed = mode_controller.tick();

    if mode_changed {
        // Reverse all active ghosts
        for (state, mut velocity) in ghost_query.iter_mut() {
            if state.is_active() {
                velocity.direction = velocity.direction.opposite();
            }
        }
    }
}

/// System to manage ghost house exits based on dot counters and timers
pub fn ghost_house_system(
    mut house_controller: ResMut<GhostHouseController>,
    mut ghost_query: Query<(&GhostType, &mut GhostState)>,
) {
    // Tick the no-dot timer and check if we should force out a ghost
    if let Some(force_out_idx) = house_controller.tick() {
        // Force out the preferred ghost (0=Pinky, 1=Inky, 2=Clyde)
        let ghost_to_release = match force_out_idx {
            0 => GhostType::Pinky,
            1 => GhostType::Inky,
            2 => GhostType::Clyde,
            _ => return,
        };

        // Find and release the ghost
        for (ghost_type, mut state) in ghost_query.iter_mut() {
            if *ghost_type == ghost_to_release && state.is_in_house() {
                *state = GhostState::Exiting { progress: 0.0 };
                house_controller.on_ghost_exit(*ghost_type);
                break;
            }
        }
    }

    // Check if any ghosts should exit based on dot counters
    for (ghost_type, mut state) in ghost_query.iter_mut() {
        if matches!(*state, GhostState::InHouse { .. }) && house_controller.should_exit(*ghost_type) {
            *state = GhostState::Exiting { progress: 0.0 };
            house_controller.on_ghost_exit(*ghost_type);
        }
    }
}

/// System to update Elroy state based on remaining pellets
pub fn elroy_system(
    pellet_count: Res<PelletCount>,
    mode_controller: Res<GhostModeController>,
    mut blinky_query: Query<&mut Elroy, With<BlinkyMarker>>,
) {
    let Ok(mut elroy) = blinky_query.single_mut() else {
        return;
    };

    if elroy.suspended {
        return;
    }

    let (threshold_1, threshold_2) = elroy_thresholds(mode_controller.level);
    // Classic Pac-Man has 244 pellets total (240 regular + 4 power)
    const TOTAL_PELLETS: u32 = 244;
    let remaining = TOTAL_PELLETS.saturating_sub(pellet_count.count());

    elroy.stage = if remaining <= threshold_2 {
        ElroyStage::Stage2
    } else if remaining <= threshold_1 {
        ElroyStage::Stage1
    } else {
        ElroyStage::None
    };
}

/// System that handles eaten ghost behavior and respawn logic.
///
/// When a ghost is eaten by Pac-Man, it enters an "Eyes" state where:
/// 1. It displays eyes-only animation
/// 2. It moves directly back to the ghost house at increased speed
/// 3. Once it reaches the ghost house center, it transitions to Reviving state
///
/// This system runs after the main movement system to override eaten ghost movement.
pub fn eaten_ghost_system(
    map: Res<Map>,
    delta_time: Res<DeltaTime>,
    mut eaten_ghosts: Query<(&GhostType, &mut Position, &mut Velocity, &mut GhostState), Without<Frozen>>,
) {
    for (ghost_type, mut position, mut velocity, mut ghost_state) in eaten_ghosts.iter_mut() {
        // Only process ghosts that are in Eyes state
        if !matches!(*ghost_state, GhostState::Eyes) {
            continue;
        }

        // Set higher speed for eaten ghosts returning to ghost house
        let original_speed = velocity.speed;
        velocity.speed = ghost_type.base_speed() * 2.0; // Move twice as fast when eaten

        // Calculate direction towards ghost house center (using Clyde's start position)
        let ghost_house_center = map.start_positions.clyde;

        match *position {
            Position::Stopped { node: current_node } => {
                // Find path to ghost house center and start moving
                if let Some((direction, edge)) = find_direction_to_target(&map, current_node, ghost_house_center) {
                    velocity.direction = direction;
                    *position = Position::Moving {
                        from: current_node,
                        to: edge.target,
                        remaining_distance: edge.distance,
                    };
                }
            }
            Position::Moving { to, .. } => {
                let distance = velocity.speed * constants::TICKS_PER_SECOND * delta_time.seconds;
                if let Some(_overflow) = position.tick(distance) {
                    // Reached target node, check if we're at ghost house center
                    if to == ghost_house_center {
                        // Transition to Reviving state
                        debug!(ghost = ?ghost_type, "Eaten ghost reached ghost house, entering reviving state");
                        *ghost_state = GhostState::Reviving {
                            remaining_ticks: constants::mechanics::GHOST_REVIVE_TICKS,
                        };
                        // Reset to stopped at ghost house center
                        *position = Position::Stopped {
                            node: ghost_house_center,
                        };
                    } else {
                        // Continue pathfinding to ghost house
                        if let Some((next_direction, edge)) = find_direction_to_target(&map, to, ghost_house_center) {
                            velocity.direction = next_direction;
                            *position = Position::Moving {
                                from: to,
                                to: edge.target,
                                remaining_distance: edge.distance,
                            };
                        }
                    }
                }
            }
        }

        // Restore original speed
        velocity.speed = original_speed;
    }
}

/// Helper function to find the direction and edge from a node towards a target node.
/// Uses simple greedy pathfinding - prefers straight lines when possible.
fn find_direction_to_target(map: &Map, from_node: NodeId, target_node: NodeId) -> Option<(Direction, Edge)> {
    let from_pos = map.graph.get_node(from_node)?.position;
    let target_pos = map.graph.get_node(target_node)?.position;

    let dx = target_pos.x as i32 - from_pos.x as i32;
    let dy = target_pos.y as i32 - from_pos.y as i32;

    // Prefer horizontal movement first, then vertical
    let preferred_dirs = if dx.abs() > dy.abs() {
        if dx > 0 {
            [Direction::Right, Direction::Up, Direction::Down, Direction::Left]
        } else {
            [Direction::Left, Direction::Up, Direction::Down, Direction::Right]
        }
    } else if dy > 0 {
        [Direction::Down, Direction::Left, Direction::Right, Direction::Up]
    } else {
        [Direction::Up, Direction::Left, Direction::Right, Direction::Down]
    };

    // Return first available direction towards target
    for direction in preferred_dirs {
        if let Some(edge) = map.graph.adjacency_list[from_node as usize].get(direction) {
            if edge.traversal_flags.contains(TraversalFlags::GHOST) {
                return Some((direction, edge));
            }
        }
    }

    None
}

/// Unified system that manages ghost state transitions and animations with component swapping
pub fn ghost_state_system(
    mut commands: Commands,
    animations: Res<GhostAnimations>,
    mut ghosts: Query<(Entity, &GhostType, &mut GhostState, &mut LastAnimationState)>,
) {
    for (entity, ghost_type, mut ghost_state, mut last_animation_state) in ghosts.iter_mut() {
        // Tick the ghost state to handle internal transitions (like flashing, reviving)
        let _ = ghost_state.tick();

        // Only update animation if the animation state actually changed
        let current_animation_state = ghost_state.animation_state();
        if last_animation_state.0 != current_animation_state {
            trace!(ghost = ?ghost_type, old_state = ?last_animation_state.0, new_state = ?current_animation_state, "Ghost animation state changed");
            match current_animation_state {
                GhostAnimationState::Frightened { flash } => {
                    // Remove DirectionalAnimation, add LinearAnimation with Looping component
                    commands
                        .entity(entity)
                        .remove::<DirectionalAnimation>()
                        .insert(animations.frightened(flash).clone())
                        .insert(Looping);
                }
                GhostAnimationState::Normal => {
                    // Remove LinearAnimation and Looping, add DirectionalAnimation
                    commands.entity(entity).remove::<(LinearAnimation, Looping)>().insert(
                        animations
                            .get_normal(ghost_type)
                            .expect("ghost type must have normal animation")
                            .clone(),
                    );
                }
                GhostAnimationState::Eyes => {
                    // Remove LinearAnimation and Looping, add DirectionalAnimation (eyes animation)
                    trace!(ghost = ?ghost_type, "Switching to eyes animation for eaten ghost");
                    commands
                        .entity(entity)
                        .remove::<(LinearAnimation, Looping)>()
                        .insert(animations.eyes().clone());
                }
            }
            last_animation_state.0 = current_animation_state;
        }
    }
}
