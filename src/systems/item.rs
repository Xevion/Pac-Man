use bevy_ecs::{
    entity::Entity,
    event::{EventReader, EventWriter},
    query::With,
    system::{Commands, Query, Res, ResMut},
};

use crate::{
    events::GameEvent,
    systems::{AudioEvent, CombatState, EntityType, ItemCollider, LevelTiming, PacmanCollider, ScoreResource},
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
    mut combat_q: Query<&mut CombatState, With<PacmanCollider>>,
    item_query: Query<(Entity, &EntityType), With<ItemCollider>>,
    mut events: EventWriter<AudioEvent>,
    level_timing: Res<LevelTiming>,
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

                    // Activate energizer on power pellet using tick-based durations
                    if *entity_type == EntityType::PowerPellet {
                        if let Ok(mut combat) = combat_q.single_mut() {
                            // Convert seconds to frames (assumes 60 FPS)
                            let total_ticks = (level_timing.energizer_duration * 60.0).round().clamp(0.0, u32::MAX as f32) as u32;
                            // Flash lead: e.g., 3 seconds (180 ticks) before end; ensure it doesn't underflow
                            let flash_lead_ticks = (level_timing.energizer_flash_threshold * 60.0)
                                .round()
                                .clamp(0.0, u32::MAX as f32) as u32;
                            combat.activate_energizer_ticks(total_ticks, flash_lead_ticks);
                        }
                    }
                }
            }
        }
    }
}
