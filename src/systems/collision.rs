use bevy_ecs::entity::Entity;
use bevy_ecs::event::EventWriter;
use bevy_ecs::query::With;
use bevy_ecs::system::{Query, Res};

use crate::error::GameError;
use crate::events::GameEvent;
use crate::map::builder::Map;
use crate::systems::components::{Collider, ItemCollider, PacmanCollider};
use crate::systems::movement::Position;

pub fn collision_system(
    map: Res<Map>,
    pacman_query: Query<(Entity, &Position, &Collider), With<PacmanCollider>>,
    item_query: Query<(Entity, &Position, &Collider), With<ItemCollider>>,
    mut events: EventWriter<GameEvent>,
    mut errors: EventWriter<GameError>,
) {
    // Check PACMAN × ITEM collisions
    for (pacman_entity, pacman_pos, pacman_collider) in pacman_query.iter() {
        for (item_entity, item_pos, item_collider) in item_query.iter() {
            match (
                pacman_pos.get_pixel_position(&map.graph),
                item_pos.get_pixel_position(&map.graph),
            ) {
                (Ok(pacman_pixel), Ok(item_pixel)) => {
                    // Calculate the distance between the two entities's precise pixel positions
                    let distance = pacman_pixel.distance(item_pixel);
                    // Calculate the distance at which the two entities will collide
                    let collision_distance = (pacman_collider.size + item_collider.size) / 2.0;

                    // If the distance between the two entities is less than the collision distance, then the two entities are colliding
                    if distance < collision_distance {
                        events.write(GameEvent::Collision(pacman_entity, item_entity));
                    }
                }
                // Either or both of the pixel positions failed to get, so we need to report the error
                (result_a, result_b) => {
                    for result in [result_a, result_b] {
                        if let Err(e) = result {
                            errors.write(GameError::InvalidState(format!(
                                "Collision system failed to get pixel positions for entities {:?} and {:?}: {}",
                                pacman_entity, item_entity, e
                            )));
                        }
                    }
                }
            }
        }
    }
}
