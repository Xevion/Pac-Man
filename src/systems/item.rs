use bevy_ecs::{
    entity::Entity,
    event::{EventReader, EventWriter},
    query::With,
    system::{Commands, Query, ResMut, Single},
};
use tracing::{debug, trace};

use crate::{
    constants::animation::FRIGHTENED_FLASH_START_TICKS,
    events::GameEvent,
    systems::{AudioEvent, EntityType, GhostCollider, GhostState, ItemCollider, PacmanCollider, ScoreResource},
};

/// Determines if a collision between two entity types should be handled by the item system.
///
/// Returns `true` if one entity is a player and the other is a collectible item.
#[allow(dead_code)]
pub fn is_valid_item_collision(entity1: EntityType, entity2: EntityType) -> bool {
    match (entity1, entity2) {
        (EntityType::Player, entity) | (entity, EntityType::Player) => entity.is_collectible(),
        _ => false,
    }
}

pub fn item_system(
    mut commands: Commands,
    mut collision_events: EventReader<GameEvent>,
    mut score: ResMut<ScoreResource>,
    pacman: Single<Entity, With<PacmanCollider>>,
    item_query: Query<(Entity, &EntityType), With<ItemCollider>>,
    mut ghost_query: Query<&mut GhostState, With<GhostCollider>>,
    mut events: EventWriter<AudioEvent>,
) {
    for event in collision_events.read() {
        if let GameEvent::Collision(entity1, entity2) = event {
            // Check if one is Pacman and the other is an item
            let (_, item_entity) = if *pacman == *entity1 && item_query.get(*entity2).is_ok() {
                (*pacman, *entity2)
            } else if *pacman == *entity2 && item_query.get(*entity1).is_ok() {
                (*pacman, *entity1)
            } else {
                continue;
            };

            // Get the item type and update score
            if let Ok((item_ent, entity_type)) = item_query.get(item_entity) {
                if let Some(score_value) = entity_type.score_value() {
                    trace!(item_entity = ?item_ent, item_type = ?entity_type, score_value, new_score = score.0 + score_value, "Item collected by player");
                    score.0 += score_value;

                    // Remove the collected item
                    commands.entity(item_ent).despawn();

                    // Trigger audio if appropriate
                    if entity_type.is_collectible() {
                        events.write(AudioEvent::PlayEat);
                    }

                    // Make ghosts frightened when power pellet is collected
                    if *entity_type == EntityType::PowerPellet {
                        // Convert seconds to frames (assumes 60 FPS)
                        let total_ticks = 60 * 5; // 5 seconds total
                        debug!(duration_ticks = total_ticks, "Power pellet collected, frightening ghosts");

                        // Set all ghosts to frightened state, except those in Eyes state
                        let mut frightened_count = 0;
                        for mut ghost_state in ghost_query.iter_mut() {
                            if !matches!(*ghost_state, GhostState::Eyes) {
                                *ghost_state = GhostState::new_frightened(total_ticks, FRIGHTENED_FLASH_START_TICKS);
                                frightened_count += 1;
                            }
                        }
                        debug!(frightened_count, "Ghosts set to frightened state");
                    }
                }
            }
        }
    }
}
