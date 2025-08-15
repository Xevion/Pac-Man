use bevy_ecs::{event::EventReader, prelude::*, query::With, system::Query};

use crate::{
    events::GameEvent,
    systems::components::{EntityType, ItemCollider, PacmanCollider, ScoreResource},
};

pub fn item_system(
    mut commands: Commands,
    mut collision_events: EventReader<GameEvent>,
    mut score: ResMut<ScoreResource>,
    pacman_query: Query<Entity, With<PacmanCollider>>,
    item_query: Query<(Entity, &EntityType), With<ItemCollider>>,
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
                match entity_type {
                    EntityType::Pellet => {
                        score.0 += 10;
                    }
                    EntityType::PowerPellet => {
                        score.0 += 50;
                    }
                    _ => continue,
                }

                // Remove the collected item
                commands.entity(item_ent).despawn();
            }
        }
    }
}
