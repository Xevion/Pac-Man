use bevy_ecs::entity::Entity;
use bevy_ecs::event::{EventReader, EventWriter};
use bevy_ecs::query::{With, Without};
use bevy_ecs::system::{Commands, Query, Res, ResMut};
use rand::rngs::SmallRng;
use rand::seq::IndexedRandom;
use rand::SeedableRng;
use smallvec::SmallVec;

use crate::events::GameEvent;
use crate::systems::audio::AudioEvent;
use crate::systems::components::{Frozen, GhostCollider, ScoreResource};
use crate::{
    map::{
        builder::Map,
        direction::Direction,
        graph::{Edge, TraversalFlags},
    },
    systems::{
        components::{CombatState, DeltaTime, Ghost, PlayerControlled},
        movement::{Position, Velocity},
    },
};

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

pub fn ghost_collision_system(
    mut commands: Commands,
    mut collision_events: EventReader<GameEvent>,
    mut score: ResMut<ScoreResource>,
    pacman_query: Query<&CombatState, With<PlayerControlled>>,
    ghost_query: Query<(Entity, &Ghost), With<GhostCollider>>,
    mut events: EventWriter<AudioEvent>,
) {
    for event in collision_events.read() {
        if let GameEvent::Collision(entity1, entity2) = event {
            // Check if one is Pacman and the other is a ghost
            let (pacman_entity, ghost_entity) = if pacman_query.get(*entity1).is_ok() && ghost_query.get(*entity2).is_ok() {
                (*entity1, *entity2)
            } else if pacman_query.get(*entity2).is_ok() && ghost_query.get(*entity1).is_ok() {
                (*entity2, *entity1)
            } else {
                continue;
            };

            // Check if Pac-Man is energized
            if let Ok(combat_state) = pacman_query.get(pacman_entity) {
                if combat_state.is_energized() {
                    // Pac-Man eats the ghost
                    if let Ok((ghost_ent, _ghost_type)) = ghost_query.get(ghost_entity) {
                        // Add score (200 points per ghost eaten)
                        score.0 += 200;

                        // Remove the ghost
                        commands.entity(ghost_ent).despawn();

                        // Play eat sound
                        events.write(AudioEvent::PlayEat);
                    }
                } else {
                    // Pac-Man dies (this would need a death system)
                    // For now, just log it
                    tracing::warn!("Pac-Man collided with ghost while not energized!");
                }
            }
        }
    }
}
