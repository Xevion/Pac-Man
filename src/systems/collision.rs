use bevy_ecs::entity::Entity;
use bevy_ecs::event::EventWriter;
use bevy_ecs::query::With;
use bevy_ecs::system::{Query, Res};

use crate::error::GameError;
use crate::events::GameEvent;
use crate::map::builder::Map;
use crate::systems::components::{Collider, GhostCollider, ItemCollider, PacmanCollider};
use crate::systems::movement::Position;

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
