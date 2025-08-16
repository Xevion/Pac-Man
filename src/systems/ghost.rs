use bevy_ecs::system::{Query, Res};
use rand::prelude::*;
use smallvec::SmallVec;

use crate::{
    entity::{direction::Direction, graph::Edge},
    map::builder::Map,
    systems::{
        components::{DeltaTime, Ghost},
        movement::{Position, Velocity},
    },
};

/// Ghost AI system that handles randomized movement decisions.
///
/// This system runs on all ghosts and makes periodic decisions about
/// which direction to move in when they reach intersections.
pub fn ghost_movement_system(
    map: Res<Map>,
    delta_time: Res<DeltaTime>,
    mut ghosts: Query<(&mut Ghost, &mut Velocity, &mut Position)>,
) {
    for (mut ghost, mut velocity, mut position) in ghosts.iter_mut() {
        let mut distance = velocity.speed * 60.0 * delta_time.0;
        loop {
            match *position {
                Position::Stopped { node: current_node } => {
                    let intersection = &map.graph.adjacency_list[current_node];
                    let opposite = velocity.direction.opposite();

                    let mut non_opposite_options: SmallVec<[Edge; 3]> = SmallVec::new();

                    // Collect all available directions that ghosts can traverse
                    for edge in Direction::DIRECTIONS.iter().flat_map(|d| intersection.get(*d)) {
                        if edge.traversal_flags.contains(crate::entity::graph::TraversalFlags::GHOST) {
                            if edge.direction != opposite {
                                non_opposite_options.push(edge);
                            }
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

/// Chooses a random available direction for a ghost at an intersection.
///
/// This function mirrors the behavior from the old ghost implementation,
/// preferring not to reverse direction unless it's the only option.
fn choose_random_direction(map: &Map, velocity: &mut Velocity, position: &Position) {
    let current_node = position.current_node();
    let intersection = &map.graph.adjacency_list[current_node];

    // Collect all available directions that ghosts can traverse
    let mut available_directions = SmallVec::<[Direction; 4]>::new();
    for direction in Direction::DIRECTIONS {
        if let Some(edge) = intersection.get(direction) {
            // Check if ghosts can traverse this edge
            if edge.traversal_flags.contains(crate::entity::graph::TraversalFlags::GHOST) {
                available_directions.push(direction);
            }
        }
    }

    // Choose a random direction (avoid reversing unless necessary)
    if !available_directions.is_empty() {
        let mut rng = SmallRng::from_os_rng();

        // Filter out the opposite direction if possible, but allow it if we have limited options
        let opposite = velocity.direction.opposite();
        let filtered_directions: Vec<_> = available_directions
            .iter()
            .filter(|&&dir| dir != opposite || available_directions.len() <= 2)
            .collect();

        if let Some(&random_direction) = filtered_directions.choose(&mut rng) {
            velocity.direction = *random_direction;
        }
    }
}
