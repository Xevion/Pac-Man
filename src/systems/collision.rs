use bevy_ecs::{
    component::Component,
    entity::Entity,
    event::EventWriter,
    observer::Trigger,
    query::With,
    system::{Commands, Query, Res, ResMut},
};
use tracing::{debug, trace, warn};

use crate::{
    constants,
    systems::{movement::Position, AudioEvent, DyingSequence, GameStage, Ghost, ScoreResource, SpawnTrigger},
};
use crate::{error::GameError, systems::GhostState};
use crate::{
    events::{CollisionTrigger, StageTransition},
    systems::PelletCount,
};
use crate::{map::builder::Map, systems::EntityType};

/// A component for defining the collision area of an entity.
#[derive(Component)]
pub struct Collider {
    pub size: f32,
}

impl Collider {
    /// Checks if this collider collides with another collider at the given distance.
    pub fn collides_with(&self, other_size: f32, distance: f32) -> bool {
        let collision_distance = (self.size + other_size) / 2.0;
        distance < collision_distance
    }
}

/// Marker components for collision filtering optimization
#[derive(Component)]
pub struct PacmanCollider;

#[derive(Component)]
pub struct GhostCollider;

#[derive(Component)]
pub struct ItemCollider;

/// Helper function to check collision between two entities with colliders.
pub fn check_collision(
    pos1: &Position,
    collider1: &Collider,
    pos2: &Position,
    collider2: &Collider,
    map: &Map,
) -> Result<bool, GameError> {
    let pixel1 = pos1
        .get_pixel_position(&map.graph)
        .map_err(|e| GameError::InvalidState(format!("Failed to get pixel position for entity 1: {}", e)))?;
    let pixel2 = pos2
        .get_pixel_position(&map.graph)
        .map_err(|e| GameError::InvalidState(format!("Failed to get pixel position for entity 2: {}", e)))?;

    let distance = pixel1.distance(pixel2);
    Ok(collider1.collides_with(collider2.size, distance))
}

/// Detects overlapping entities and triggers collision observers immediately.
///
/// Performs distance-based collision detection between Pac-Man and collectible items
/// using each entity's position and collision radius. When entities overlap, triggers
/// collision observers for immediate handling without race conditions.
/// Collision detection accounts for both entities being in motion and supports
/// circular collision boundaries for accurate gameplay feel.
///
/// Also detects collisions between Pac-Man and ghosts for gameplay mechanics like
/// power pellet effects, ghost eating, and player death.
#[allow(clippy::too_many_arguments)]
pub fn collision_system(
    map: Res<Map>,
    pacman_query: Query<(Entity, &Position, &Collider), With<PacmanCollider>>,
    item_query: Query<(Entity, &Position, &Collider), With<ItemCollider>>,
    ghost_query: Query<(Entity, &Position, &Collider, &Ghost, &GhostState), With<GhostCollider>>,
    mut commands: Commands,
    mut errors: EventWriter<GameError>,
) {
    // Check PACMAN × ITEM collisions
    for (pacman_entity, pacman_pos, pacman_collider) in pacman_query.iter() {
        for (item_entity, item_pos, item_collider) in item_query.iter() {
            match check_collision(pacman_pos, pacman_collider, item_pos, item_collider, &map) {
                Ok(colliding) => {
                    if colliding {
                        trace!("Item collision detected");
                        commands.trigger(CollisionTrigger::ItemCollision { item: item_entity });
                    }
                }
                Err(e) => {
                    errors.write(GameError::InvalidState(format!(
                        "Collision system failed to check collision between entities {:?} and {:?}: {}",
                        pacman_entity, item_entity, e
                    )));
                }
            }
        }

        // Check PACMAN × GHOST collisions
        for (ghost_entity, ghost_pos, ghost_collider, ghost, ghost_state) in ghost_query.iter() {
            match check_collision(pacman_pos, pacman_collider, ghost_pos, ghost_collider, &map) {
                Ok(colliding) => {
                    if !colliding || matches!(*ghost_state, GhostState::Eyes) {
                        continue;
                    }

                    trace!(ghost = ?ghost, "Ghost collision detected");
                    commands.trigger(CollisionTrigger::GhostCollision {
                        pacman: pacman_entity,
                        ghost: ghost_entity,
                        ghost_type: *ghost,
                    });
                }
                Err(e) => {
                    errors.write(GameError::InvalidState(format!(
                        "Collision system failed to check collision between entities {:?} and {:?}: {}",
                        pacman_entity, ghost_entity, e
                    )));
                }
            }
        }
    }
}

/// Observer for handling ghost collisions immediately when they occur
#[allow(clippy::too_many_arguments)]
pub fn ghost_collision_observer(
    trigger: Trigger<CollisionTrigger>,
    mut stage_events: EventWriter<StageTransition>,
    mut score: ResMut<ScoreResource>,
    mut game_state: ResMut<GameStage>,
    mut ghost_state_query: Query<&mut GhostState>,
    mut events: EventWriter<AudioEvent>,
) {
    if let CollisionTrigger::GhostCollision {
        pacman: _pacman,
        ghost,
        ghost_type,
    } = *trigger
    {
        // Check if Pac-Man is already dying
        if matches!(*game_state, GameStage::PlayerDying(_)) {
            return;
        }

        // Check if the ghost is frightened
        if let Ok(mut ghost_state) = ghost_state_query.get_mut(ghost) {
            // Check if ghost is in frightened state
            if matches!(*ghost_state, GhostState::Frightened { .. }) {
                // Pac-Man eats the ghost
                // Add score (200 points per ghost eaten)
                debug!(ghost = ?ghost_type, score_added = 200, new_score = score.0 + 200, "Pacman ate frightened ghost");
                score.0 += 200;

                *ghost_state = GhostState::Eyes;

                // Enter short pause to show bonus points, hide ghost, then set Eyes after pause
                // Request transition via event so stage_system can process it
                stage_events.write(StageTransition::GhostEatenPause {
                    ghost_entity: ghost,
                    ghost_type: ghost_type,
                });

                // Play eat sound
                events.write(AudioEvent::PlayEat);
            } else if matches!(*ghost_state, GhostState::Normal) {
                // Pac-Man dies
                warn!(ghost = ?ghost_type, "Pacman hit by normal ghost, player dies");
                *game_state = GameStage::PlayerDying(DyingSequence::Frozen { remaining_ticks: 60 });
                events.write(AudioEvent::StopAll);
            } else {
                trace!(ghost_state = ?*ghost_state, "Ghost collision ignored due to state");
            }
        }
    }
}

/// Observer for handling item collisions immediately when they occur
#[allow(clippy::too_many_arguments)]
pub fn item_collision_observer(
    trigger: Trigger<CollisionTrigger>,
    mut commands: Commands,
    mut score: ResMut<ScoreResource>,
    mut pellet_count: ResMut<PelletCount>,
    item_query: Query<(Entity, &EntityType, &Position), With<ItemCollider>>,
    mut ghost_query: Query<&mut GhostState, With<GhostCollider>>,
    mut events: EventWriter<AudioEvent>,
) {
    if let CollisionTrigger::ItemCollision { item } = *trigger {
        // Get the item type and update score
        if let Ok((item_ent, entity_type, position)) = item_query.get(item) {
            if let Some(score_value) = entity_type.score_value() {
                trace!(item_entity = ?item_ent, item_type = ?entity_type, score_value, new_score = score.0 + score_value, "Item collected by player");
                score.0 += score_value;

                // Remove the collected item
                commands.entity(item_ent).despawn();

                // Track pellet consumption for fruit spawning
                if *entity_type == EntityType::Pellet {
                    pellet_count.0 += 1;
                    trace!(pellet_count = pellet_count.0, "Pellet consumed");

                    // Check if we should spawn a fruit
                    if pellet_count.0 == 5 || pellet_count.0 == 170 {
                        debug!(pellet_count = pellet_count.0, "Fruit spawn milestone reached");
                        commands.trigger(SpawnTrigger::Fruit);
                    }
                }

                // Trigger bonus points effect if a fruit is collected
                if matches!(*entity_type, EntityType::Fruit(_)) {
                    commands.trigger(SpawnTrigger::Bonus {
                        position: *position,
                        value: entity_type.score_value().unwrap(),
                        ttl: 60 * 2,
                    });
                }

                // Trigger audio if appropriate
                if entity_type.is_collectible() {
                    events.write(AudioEvent::PlayEat);
                }

                // Make non-eaten ghosts frightened when power pellet is collected
                if matches!(*entity_type, EntityType::PowerPellet) {
                    debug!(
                        duration_ticks = constants::animation::GHOST_FRIGHTENED_TICKS,
                        "Power pellet collected, frightening ghosts"
                    );
                    for mut ghost_state in ghost_query.iter_mut() {
                        if matches!(*ghost_state, GhostState::Normal) {
                            *ghost_state = GhostState::new_frightened(
                                constants::animation::GHOST_FRIGHTENED_TICKS,
                                constants::animation::GHOST_FRIGHTENED_FLASH_START_TICKS,
                            );
                        }
                    }
                    debug!(
                        frightened_count = ghost_query.iter().count(),
                        "Ghosts set to frightened state"
                    );
                }
            }
        }
    }
}
