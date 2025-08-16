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
    mut ghosts: Query<(&Ghost, &mut Velocity, &mut Position)>,
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
                        if edge.traversal_flags.contains(crate::entity::graph::TraversalFlags::GHOST)
                            && edge.direction != opposite
                        {
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
