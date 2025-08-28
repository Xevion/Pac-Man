use bevy_ecs::{
    entity::Entity,
    event::{EventReader, EventWriter},
    query::With,
    system::{Commands, Query, Res, ResMut},
};

use crate::{
    events::GameEvent,
    systems::{AudioEvent, EntityType, GhostCollider, ItemCollider, PacmanCollider, ScoreResource, Vulnerable},
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
    pacman_query: Query<Entity, With<PacmanCollider>>,
    item_query: Query<(Entity, &EntityType), With<ItemCollider>>,
    ghost_query: Query<Entity, With<GhostCollider>>,
    mut events: EventWriter<AudioEvent>,
) {
    for event in collision_events.read() {
        if let GameEvent::Collision(entity1, entity2) = event {
            // Check if one is Pacman and the other is an item
            let (_pacman_entity, item_entity) = if pacman_query.get(*entity1).is_ok() && item_query.get(*entity2).is_ok() {
                (*entity1, *entity2)
            } else if pacman_query.get(*entity2).is_ok() && item_query.get(*entity1).is_ok() {
                (*entity2, *entity1)
            } else {
                continue;
            };

            // Get the item type and update score
            if let Ok((item_ent, entity_type)) = item_query.get(item_entity) {
                if let Some(score_value) = entity_type.score_value() {
                    score.0 += score_value;

                    // Remove the collected item
                    commands.entity(item_ent).despawn();

                    // Trigger audio if appropriate
                    if entity_type.is_collectible() {
                        events.write(AudioEvent::PlayEat);
                    }

                    // Make ghosts vulnerable when power pellet is collected
                    if *entity_type == EntityType::PowerPellet {
                        // Convert seconds to frames (assumes 60 FPS)
                        let total_ticks = 60 * 5;

                        // Add Vulnerable component to all ghosts
                        for ghost_entity in ghost_query.iter() {
                            commands.entity(ghost_entity).insert(Vulnerable {
                                remaining_ticks: total_ticks,
                            });
                        }
                    }
                }
            }
        }
    }
}
