use bevy_ecs::{
    entity::Entity,
    event::{Event, EventReader, EventWriter},
    observer::Trigger,
    query::With,
    system::{Commands, NonSendMut, Query, Res, ResMut, Single},
};
use tracing::{debug, trace};

use crate::{
    constants::collider::FRUIT_SIZE,
    map::builder::Map,
    systems::{common::bundles::ItemBundle, Collider, Position, Renderable},
    texture::{sprite::SpriteAtlas, sprites::GameSprite},
};

use crate::{
    constants::animation::FRIGHTENED_FLASH_START_TICKS,
    events::GameEvent,
    systems::common::components::EntityType,
    systems::lifetime::TimeToLive,
    systems::{AudioEvent, GhostCollider, GhostState, ItemCollider, LinearAnimation, PacmanCollider, ScoreResource},
    texture::animated::TileSequence,
};

/// Tracks the number of pellets consumed by the player for fruit spawning mechanics.
#[derive(bevy_ecs::resource::Resource, Debug, Default)]
pub struct PelletCount(pub u32);

/// Maps fruit score values to bonus sprite indices for displaying bonus points
fn fruit_score_to_sprite_index(score: u32) -> u8 {
    match score {
        100 => 0,   // Cherry
        300 => 2,   // Strawberry
        500 => 3,   // Orange
        700 => 4,   // Apple
        1000 => 6,  // Melon
        2000 => 8,  // Galaxian
        3000 => 9,  // Bell
        5000 => 10, // Key
        _ => 0,     // Default to 100 points sprite
    }
}

/// Maps sprite index to the corresponding effect sprite path (same as in state.rs)
fn sprite_index_to_path(index: u8) -> &'static str {
    match index {
        0 => "effects/100.png",
        1 => "effects/200.png",
        2 => "effects/300.png",
        3 => "effects/400.png",
        4 => "effects/700.png",
        5 => "effects/800.png",
        6 => "effects/1000.png",
        7 => "effects/1600.png",
        8 => "effects/2000.png",
        9 => "effects/3000.png",
        10 => "effects/5000.png",
        _ => "effects/100.png", // fallback to index 0
    }
}

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

#[allow(clippy::too_many_arguments)]
pub fn item_system(
    mut commands: Commands,
    mut collision_events: EventReader<GameEvent>,
    mut score: ResMut<ScoreResource>,
    mut pellet_count: ResMut<PelletCount>,
    pacman: Single<Entity, With<PacmanCollider>>,
    item_query: Query<(Entity, &EntityType, &Position), With<ItemCollider>>,
    mut ghost_query: Query<&mut GhostState, With<GhostCollider>>,
    mut events: EventWriter<AudioEvent>,
    atlas: NonSendMut<SpriteAtlas>,
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
            if let Ok((item_ent, entity_type, item_position)) = item_query.get(item_entity) {
                if let Some(score_value) = entity_type.score_value() {
                    trace!(item_entity = ?item_ent, item_type = ?entity_type, score_value, new_score = score.0 + score_value, "Item collected by player");
                    score.0 += score_value;

                    // Spawn bonus sprite for fruits at the fruit's position (similar to ghost eating bonus)
                    if matches!(entity_type, EntityType::Fruit(_)) {
                        let sprite_index = fruit_score_to_sprite_index(score_value);
                        let sprite_path = sprite_index_to_path(sprite_index);

                        if let Ok(sprite_tile) = SpriteAtlas::get_tile(&atlas, sprite_path) {
                            let tile_sequence = TileSequence::single(sprite_tile);
                            let animation = LinearAnimation::new(tile_sequence, 1);

                            commands.spawn((
                                *item_position,
                                Renderable {
                                    sprite: sprite_tile,
                                    layer: 2, // Above other entities
                                },
                                animation,
                                TimeToLive::new(120), // 2 seconds at 60 FPS
                            ));

                            debug!(
                                fruit_score = score_value,
                                sprite_index, "Fruit bonus sprite spawned at fruit position"
                            );
                        }
                    }

                    // Remove the collected item
                    commands.entity(item_ent).despawn();

                    // Track pellet consumption for fruit spawning
                    if *entity_type == EntityType::Pellet {
                        pellet_count.0 += 1;
                        trace!(pellet_count = pellet_count.0, "Pellet consumed");

                        // Check if we should spawn a fruit
                        if pellet_count.0 == 70 || pellet_count.0 == 170 {
                            debug!(pellet_count = pellet_count.0, "Fruit spawn milestone reached");
                            commands.trigger(SpawnFruitTrigger);
                        }
                    }

                    // Trigger audio if appropriate
                    if entity_type.is_collectible() {
                        events.write(AudioEvent::PlayEat);
                    }

                    // Make ghosts frightened when power pellet is collected
                    if matches!(*entity_type, EntityType::PowerPellet) {
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

/// Trigger to spawn a fruit
#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpawnFruitTrigger;

pub fn spawn_fruit_observer(
    _: Trigger<SpawnFruitTrigger>,
    mut commands: Commands,
    atlas: NonSendMut<SpriteAtlas>,
    map: Res<Map>,
) {
    // Use cherry sprite as the default fruit (first fruit in original Pac-Man)
    let fruit_sprite = &atlas
        .get_tile(&GameSprite::Fruit(crate::texture::sprites::FruitSprite::Cherry).to_path())
        .unwrap();

    let fruit_entity = commands.spawn(ItemBundle {
        position: map.start_positions.fruit_spawn,
        sprite: Renderable {
            sprite: *fruit_sprite,
            layer: 1,
        },
        entity_type: EntityType::Fruit(crate::texture::sprites::FruitSprite::Cherry),
        collider: Collider { size: FRUIT_SIZE },
        item_collider: ItemCollider,
    });

    debug!(fruit_entity = ?fruit_entity.id(), fruit_spawn_node = ?map.start_positions.fruit_spawn, "Fruit spawned");
}
