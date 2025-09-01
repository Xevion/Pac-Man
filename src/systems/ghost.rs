use crate::systems::components::Frozen;
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

use bevy_ecs::{
    query::Added,
    removal_detection::RemovedComponents,
    system::{Commands, Query, Res},
};

use crate::systems::{Eaten, GhostAnimations, Vulnerable};

use bevy_ecs::query::{With, Without};
use rand::rngs::SmallRng;
use rand::seq::IndexedRandom;
use rand::SeedableRng;
use smallvec::SmallVec;

/// Autonomous ghost AI system implementing randomized movement with backtracking avoidance.
pub fn ghost_movement_system(
    map: Res<Map>,
    delta_time: Res<DeltaTime>,
    mut ghosts: Query<(&Ghost, &mut Velocity, &mut Position), Without<Frozen>>,
) {
    for (_ghost, mut velocity, mut position) in ghosts.iter_mut() {
        let mut distance = velocity.speed * 60.0 * delta_time.0;
        loop {
            match *position {
                Position::Stopped { node: current_node } => {
                    let intersection = &map.graph.adjacency_list[current_node];
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
                            edge
                        } else {
                            break;
                        }
                    } else {
                        *non_opposite_options.choose(&mut SmallRng::from_os_rng()).unwrap()
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

/// System that manages ghost animation state transitions based on ghost behavior.
///
/// This system handles the following animation state changes:
/// - When a ghost becomes vulnerable (power pellet eaten): switches to frightened animation
/// - When a ghost is eaten by Pac-Man: switches to eaten (eyes) animation
/// - When vulnerability ends: switches back to normal animation
///
/// The system uses ECS change detection to efficiently track state transitions:
/// - `Added<Vulnerable>` detects when ghosts become frightened
/// - `Added<Eaten>` detects when ghosts are consumed
/// - `RemovedComponents<Vulnerable>` detects when fright period ends
///
/// This ensures smooth visual feedback for gameplay state changes while maintaining
/// separation between game logic and animation state.
pub fn ghost_state_animation_system(
    mut commands: Commands,
    animations: Res<GhostAnimations>,
    mut vulnerable_added: Query<(bevy_ecs::entity::Entity, &Ghost), Added<Vulnerable>>,
    mut eaten_added: Query<(bevy_ecs::entity::Entity, &Ghost), Added<Eaten>>,
    mut vulnerable_removed: RemovedComponents<Vulnerable>,
    ghosts: Query<&Ghost>,
) {
    // When a ghost becomes vulnerable, switch to the frightened animation
    for (entity, ghost_type) in vulnerable_added.iter_mut() {
        if let Some(animation_set) = animations.0.get(ghost_type) {
            if let Some(animation) = animation_set.frightened() {
                commands.entity(entity).insert(animation.clone());
            }
        }
    }

    // When a ghost is eaten, switch to the eaten animation
    for (entity, ghost_type) in eaten_added.iter_mut() {
        if let Some(animation_set) = animations.0.get(ghost_type) {
            if let Some(animation) = animation_set.eyes() {
                commands.entity(entity).insert(animation.clone());
            }
        }
    }

    // When a ghost is no longer vulnerable, switch back to the normal animation
    for entity in vulnerable_removed.read() {
        if let Ok(ghost_type) = ghosts.get(entity) {
            if let Some(animation_set) = animations.0.get(ghost_type) {
                if let Some(animation) = animation_set.normal() {
                    commands.entity(entity).insert(animation.clone());
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
    animations: Res<GhostAnimations>,
    mut commands: Commands,
    mut eaten_ghosts: Query<(bevy_ecs::entity::Entity, &Ghost, &mut Position, &mut Velocity), With<Eaten>>,
) {
    for (entity, ghost_type, mut position, mut velocity) in eaten_ghosts.iter_mut() {
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
                        to: map.graph.adjacency_list[current_node].get(direction).unwrap().target,
                        remaining_distance: map.graph.adjacency_list[current_node].get(direction).unwrap().distance,
                    };
                }
            }
            Position::Moving { to, .. } => {
                let distance = velocity.speed * 60.0 * delta_time.0;
                if let Some(_overflow) = position.tick(distance) {
                    // Reached target node, check if we're at ghost house center
                    if to == ghost_house_center {
                        // Respawn the ghost - remove Eaten component and switch to normal animation
                        commands.entity(entity).remove::<Eaten>();
                        if let Some(animation_set) = animations.0.get(ghost_type) {
                            if let Some(animation) = animation_set.normal() {
                                commands.entity(entity).insert(animation.clone());
                            }
                        }
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
                                to: map.graph.adjacency_list[to].get(next_direction).unwrap().target,
                                remaining_distance: map.graph.adjacency_list[to].get(next_direction).unwrap().distance,
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
fn find_direction_to_target(map: &Map, from_node: usize, target_node: usize) -> Option<Direction> {
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
        if let Some(edge) = map.graph.adjacency_list[from_node].get(direction) {
            if edge.traversal_flags.contains(TraversalFlags::GHOST) {
                return Some(direction);
            }
        }
    }

    None
}
