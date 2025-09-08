use crate::platform;
use crate::systems::components::{
    DirectionalAnimation, Frozen, GhostAnimation, GhostState, LastAnimationState, LinearAnimation, Looping,
};
use crate::{
    map::{
        builder::Map,
        direction::Direction,
        graph::{Edge, TraversalFlags},
    },
    systems::{
        components::{DeltaTime, Ghost},
        movement::{Position, Velocity},
    },
};
use tracing::{debug, trace, warn};

use crate::systems::GhostAnimations;
use bevy_ecs::query::Without;
use bevy_ecs::system::{Commands, Query, Res};
use rand::seq::IndexedRandom;
use smallvec::SmallVec;

/// Autonomous ghost AI system implementing randomized movement with backtracking avoidance.
pub fn ghost_movement_system(
    map: Res<Map>,
    delta_time: Res<DeltaTime>,
    mut ghosts: Query<(&Ghost, &mut Velocity, &mut Position), Without<Frozen>>,
) {
    for (_ghost, mut velocity, mut position) in ghosts.iter_mut() {
        let mut distance = velocity.speed * 60.0 * delta_time.seconds;
        loop {
            match *position {
                Position::Stopped { node: current_node } => {
                    let intersection = &map.graph.adjacency_list[current_node as usize];
                    let opposite = velocity.direction.opposite();

                    let mut non_opposite_options: SmallVec<[Edge; 3]> = SmallVec::new();

                    // Collect all available directions that ghosts can traverse
                    for edge in Direction::DIRECTIONS.iter().flat_map(|d| intersection.get(*d)) {
                        if edge.traversal_flags.contains(TraversalFlags::GHOST) && edge.direction != opposite {
                            non_opposite_options.push(edge);
                        }
                    }

                    let new_edge: Edge = if non_opposite_options.is_empty() {
                        if let Some(edge) = intersection.get(opposite) {
                            trace!(node = current_node, ghost = ?_ghost, direction = ?opposite, "Ghost forced to reverse direction");
                            edge
                        } else {
                            warn!(node = current_node, ghost = ?_ghost, "Ghost stuck with no available directions");
                            break;
                        }
                    } else {
                        *non_opposite_options.choose(&mut platform::rng()).unwrap()
                    };

                    velocity.direction = new_edge.direction;
                    *position = Position::Moving {
                        from: current_node,
                        to: new_edge.target,
                        remaining_distance: new_edge.distance,
                    };
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

/// System that handles eaten ghost behavior and respawn logic.
///
/// When a ghost is eaten by Pac-Man, it enters an "eaten" state where:
/// 1. It displays eyes-only animation
/// 2. It moves directly back to the ghost house at increased speed
/// 3. Once it reaches the ghost house center, it respawns as a normal ghost
///
/// This system runs after the main movement system to override eaten ghost movement.
pub fn eaten_ghost_system(
    map: Res<Map>,
    delta_time: Res<DeltaTime>,
    mut eaten_ghosts: Query<(&Ghost, &mut Position, &mut Velocity, &mut GhostState)>,
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
                if let Some(direction) = find_direction_to_target(&map, current_node, ghost_house_center) {
                    velocity.direction = direction;
                    *position = Position::Moving {
                        from: current_node,
                        to: map.graph.adjacency_list[current_node as usize].get(direction).unwrap().target,
                        remaining_distance: map.graph.adjacency_list[current_node as usize]
                            .get(direction)
                            .unwrap()
                            .distance,
                    };
                }
            }
            Position::Moving { to, .. } => {
                let distance = velocity.speed * 60.0 * delta_time.seconds;
                if let Some(_overflow) = position.tick(distance) {
                    // Reached target node, check if we're at ghost house center
                    if to == ghost_house_center {
                        // Respawn the ghost - set state back to normal
                        debug!(ghost = ?ghost_type, "Eaten ghost reached ghost house, respawning as normal");
                        *ghost_state = GhostState::Normal;
                        // Reset to stopped at ghost house center
                        *position = Position::Stopped {
                            node: ghost_house_center,
                        };
                    } else {
                        // Continue pathfinding to ghost house
                        if let Some(next_direction) = find_direction_to_target(&map, to, ghost_house_center) {
                            velocity.direction = next_direction;
                            *position = Position::Moving {
                                from: to,
                                to: map.graph.adjacency_list[to as usize].get(next_direction).unwrap().target,
                                remaining_distance: map.graph.adjacency_list[to as usize].get(next_direction).unwrap().distance,
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

/// Helper function to find the direction from a node towards a target node.
/// Uses simple greedy pathfinding - prefers straight lines when possible.
fn find_direction_to_target(
    map: &Map,
    from_node: crate::systems::movement::NodeId,
    target_node: crate::systems::movement::NodeId,
) -> Option<Direction> {
    let from_pos = map.graph.get_node(from_node).unwrap().position;
    let target_pos = map.graph.get_node(target_node).unwrap().position;

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
                return Some(direction);
            }
        }
    }

    None
}

/// Unified system that manages ghost state transitions and animations with component swapping
pub fn ghost_state_system(
    mut commands: Commands,
    animations: Res<GhostAnimations>,
    mut ghosts: Query<(bevy_ecs::entity::Entity, &Ghost, &mut GhostState, &mut LastAnimationState)>,
) {
    for (entity, ghost_type, mut ghost_state, mut last_animation_state) in ghosts.iter_mut() {
        // Tick the ghost state to handle internal transitions (like flashing)
        let _ = ghost_state.tick();

        // Only update animation if the animation state actually changed
        let current_animation_state = ghost_state.animation_state();
        if last_animation_state.0 != current_animation_state {
            trace!(ghost = ?ghost_type, old_state = ?last_animation_state.0, new_state = ?current_animation_state, "Ghost animation state changed");
            match current_animation_state {
                GhostAnimation::Frightened { flash } => {
                    // Remove DirectionalAnimation, add LinearAnimation with Looping component
                    commands
                        .entity(entity)
                        .remove::<DirectionalAnimation>()
                        .insert(animations.frightened(flash).clone())
                        .insert(Looping);
                }
                GhostAnimation::Normal => {
                    // Remove LinearAnimation and Looping, add DirectionalAnimation
                    commands
                        .entity(entity)
                        .remove::<(LinearAnimation, Looping)>()
                        .insert(animations.get_normal(ghost_type).unwrap().clone());
                }
                GhostAnimation::Eyes => {
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
