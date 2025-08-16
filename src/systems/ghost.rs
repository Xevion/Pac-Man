use bevy_ecs::system::{Query, Res};
use rand::prelude::*;
use smallvec::SmallVec;

use crate::{
    entity::direction::Direction,
    map::builder::Map,
    systems::{
        components::{DeltaTime, EntityType, GhostBehavior, GhostType},
        movement::{Movable, Position},
    },
};

/// Ghost AI system that handles randomized movement decisions.
///
/// This system runs on all ghosts and makes periodic decisions about
/// which direction to move in when they reach intersections.
pub fn ghost_system(
    map: Res<Map>,
    delta_time: Res<DeltaTime>,
    mut ghosts: Query<(&mut GhostBehavior, &mut Movable, &Position, &EntityType, &GhostType)>,
) {
    for (mut ghost_behavior, mut movable, position, entity_type, _ghost_type) in ghosts.iter_mut() {
        // Only process ghosts
        if *entity_type != EntityType::Ghost {
            continue;
        }

        // Update decision timer
        ghost_behavior.decision_timer += delta_time.0;

        // Check if we should make a new direction decision
        let should_decide = ghost_behavior.decision_timer >= ghost_behavior.decision_interval;
        let at_intersection = position.is_at_node();

        if should_decide && at_intersection {
            choose_random_direction(&map, &mut movable, position);
            ghost_behavior.decision_timer = 0.0;
        }
    }
}

/// Chooses a random available direction for a ghost at an intersection.
///
/// This function mirrors the behavior from the old ghost implementation,
/// preferring not to reverse direction unless it's the only option.
fn choose_random_direction(map: &Map, movable: &mut Movable, position: &Position) {
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
        let opposite = movable.current_direction.opposite();
        let filtered_directions: Vec<_> = available_directions
            .iter()
            .filter(|&&dir| dir != opposite || available_directions.len() <= 2)
            .collect();

        if let Some(&random_direction) = filtered_directions.choose(&mut rng) {
            movable.requested_direction = Some(*random_direction);
        }
    }
}
