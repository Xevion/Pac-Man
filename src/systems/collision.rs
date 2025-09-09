use bevy_ecs::{
    component::Component,
    entity::Entity,
    event::{EventReader, EventWriter},
    query::With,
    system::{Commands, Query, Res, ResMut, Single},
};
use tracing::{debug, trace, warn};

use crate::events::{GameEvent, StageTransition};
use crate::map::builder::Map;
use crate::systems::{movement::Position, AudioEvent, DyingSequence, Frozen, GameStage, Ghost, PlayerControlled, ScoreResource};
use crate::{error::GameError, systems::GhostState};

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

/// Detects overlapping entities and generates collision events for gameplay systems.
///
/// Performs distance-based collision detection between Pac-Man and collectible items
/// using each entity's position and collision radius. When entities overlap, emits
/// a `GameEvent::Collision` for the item system to handle scoring and removal.
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
    ghost_query: Query<(Entity, &Position, &Collider), With<GhostCollider>>,
    mut events: EventWriter<GameEvent>,
    mut errors: EventWriter<GameError>,
) {
    // Check PACMAN × ITEM collisions
    for (pacman_entity, pacman_pos, pacman_collider) in pacman_query.iter() {
        for (item_entity, item_pos, item_collider) in item_query.iter() {
            match check_collision(pacman_pos, pacman_collider, item_pos, item_collider, &map) {
                Ok(colliding) => {
                    if colliding {
                        trace!(pacman_entity = ?pacman_entity, item_entity = ?item_entity, "Item collision detected");
                        events.write(GameEvent::Collision(pacman_entity, item_entity));
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
        for (ghost_entity, ghost_pos, ghost_collider) in ghost_query.iter() {
            match check_collision(pacman_pos, pacman_collider, ghost_pos, ghost_collider, &map) {
                Ok(colliding) => {
                    if colliding {
                        trace!(pacman_entity = ?pacman_entity, ghost_entity = ?ghost_entity, "Ghost collision detected");
                        events.write(GameEvent::Collision(pacman_entity, ghost_entity));
                    }
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

#[allow(clippy::too_many_arguments)]
pub fn ghost_collision_system(
    mut commands: Commands,
    mut collision_events: EventReader<GameEvent>,
    mut stage_events: EventWriter<StageTransition>,
    mut score: ResMut<ScoreResource>,
    mut game_state: ResMut<GameStage>,
    player: Single<Entity, With<PlayerControlled>>,
    ghost_query: Query<(Entity, &Ghost), With<GhostCollider>>,
    mut ghost_state_query: Query<&mut GhostState>,
    mut events: EventWriter<AudioEvent>,
) {
    for event in collision_events.read() {
        if let GameEvent::Collision(entity1, entity2) = event {
            // Check if one is Pacman and the other is a ghost
            let (pacman_entity, ghost_entity) = if *entity1 == *player && ghost_query.get(*entity2).is_ok() {
                (*entity1, *entity2)
            } else if *entity2 == *player && ghost_query.get(*entity1).is_ok() {
                (*entity2, *entity1)
            } else {
                continue;
            };

            // Check if the ghost is frightened
            if let Ok((ghost_ent, _ghost_type)) = ghost_query.get(ghost_entity) {
                if let Ok(ghost_state) = ghost_state_query.get_mut(ghost_ent) {
                    // Check if ghost is in frightened state
                    if matches!(*ghost_state, GhostState::Frightened { .. }) {
                        // Pac-Man eats the ghost
                        // Add score (200 points per ghost eaten)
                        debug!(ghost_entity = ?ghost_ent, score_added = 200, new_score = score.0 + 200, "Pacman ate frightened ghost");
                        score.0 += 200;

                        // Enter short pause to show bonus points, hide ghost, then set Eyes after pause
                        // Request transition via event so stage_system can process it
                        stage_events.write(StageTransition::GhostEatenPause { ghost_entity: ghost_ent });

                        // Play eat sound
                        events.write(AudioEvent::PlayEat);
                    } else if matches!(*ghost_state, GhostState::Normal) {
                        // Pac-Man dies
                        warn!(ghost_entity = ?ghost_ent, "Pacman hit by normal ghost, player dies");
                        *game_state = GameStage::PlayerDying(DyingSequence::Frozen { remaining_ticks: 60 });
                        commands.entity(pacman_entity).insert(Frozen);
                        commands.entity(ghost_entity).insert(Frozen);
                        events.write(AudioEvent::StopAll);
                    } else {
                        trace!(ghost_state = ?*ghost_state, "Ghost collision ignored due to state");
                    }
                }
            }
        }
    }
}
