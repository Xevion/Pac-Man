use bevy_ecs::{
    entity::Entity,
    event::{Event, EventReader, EventWriter},
    observer::Trigger,
    query::With,
    system::{Commands, NonSendMut, Query, Res, ResMut, Single},
};
use strum_macros::IntoStaticStr;
use tracing::{debug, trace};

use crate::{
    constants,
    map::builder::Map,
    systems::{common::bundles::ItemBundle, Collider, Position, Renderable, TimeToLive},
    texture::{
        sprite::SpriteAtlas,
        sprites::{EffectSprite, GameSprite},
    },
};

use crate::{
    constants::animation::FRIGHTENED_FLASH_START_TICKS,
    events::GameEvent,
    systems::common::components::EntityType,
    systems::{AudioEvent, GhostCollider, GhostState, ItemCollider, PacmanCollider, ScoreResource},
};

/// Tracks the number of pellets consumed by the player for fruit spawning mechanics.
#[derive(bevy_ecs::resource::Resource, Debug, Default)]
pub struct PelletCount(pub u32);

/// Represents the different fruit sprites that can appear as bonus items.
#[derive(IntoStaticStr, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[strum(serialize_all = "snake_case")]
pub enum FruitType {
    Cherry,
    Strawberry,
    Orange,
    Apple,
    Melon,
    Galaxian,
    Bell,
    Key,
}

impl FruitType {
    /// Returns the score value for this fruit type.
    pub fn score_value(self) -> u32 {
        match self {
            FruitType::Cherry => 100,
            FruitType::Strawberry => 300,
            FruitType::Orange => 500,
            FruitType::Apple => 700,
            FruitType::Melon => 1000,
            FruitType::Galaxian => 2000,
            FruitType::Bell => 3000,
            FruitType::Key => 5000,
        }
    }

    pub fn from_index(index: u8) -> Self {
        match index {
            0 => FruitType::Cherry,
            1 => FruitType::Strawberry,
            2 => FruitType::Orange,
            3 => FruitType::Apple,
            4 => FruitType::Melon,
            5 => FruitType::Galaxian,
            6 => FruitType::Bell,
            7 => FruitType::Key,
            _ => panic!("Invalid fruit index: {}", index),
        }
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
            if let Ok((item_ent, entity_type, position)) = item_query.get(item_entity) {
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
#[derive(Event, Clone, Copy, Debug)]
pub enum SpawnTrigger {
    Fruit,
    Bonus { position: Position, value: u32, ttl: u32 },
}

pub fn spawn_fruit_observer(
    trigger: Trigger<SpawnTrigger>,
    mut commands: Commands,
    atlas: NonSendMut<SpriteAtlas>,
    map: Res<Map>,
) {
    let entity = match *trigger {
        SpawnTrigger::Fruit => {
            // Use cherry sprite as the default fruit (first fruit in original Pac-Man)
            let sprite = &atlas
                .get_tile(&GameSprite::Fruit(FruitType::from_index(0)).to_path())
                .unwrap();
            let bundle = ItemBundle {
                position: map.start_positions.fruit_spawn,
                sprite: Renderable {
                    sprite: *sprite,
                    layer: 1,
                },
                entity_type: EntityType::Fruit(FruitType::Cherry),
                collider: Collider {
                    size: constants::collider::FRUIT_SIZE,
                },
                item_collider: ItemCollider,
            };

            commands.spawn(bundle)
        }
        SpawnTrigger::Bonus { position, value, ttl } => {
            let sprite = &atlas
                .get_tile(&GameSprite::Effect(EffectSprite::Bonus(value)).to_path())
                .unwrap();

            let bundle = (
                position,
                TimeToLive::new(ttl),
                Renderable {
                    sprite: *sprite,
                    layer: 1,
                },
                EntityType::Effect,
            );

            commands.spawn(bundle)
        }
    };

    debug!(entity = ?entity.id(), "Entity spawned via trigger");
}
