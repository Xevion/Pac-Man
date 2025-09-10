use bevy_ecs::{
    event::Event,
    observer::Trigger,
    system::{Commands, NonSendMut, Res},
};
use strum_macros::IntoStaticStr;
use tracing::debug;

use crate::{
    constants,
    map::builder::Map,
    systems::{common::bundles::ItemBundle, Collider, Position, Renderable, TimeToLive},
    texture::{
        sprite::SpriteAtlas,
        sprites::{EffectSprite, GameSprite},
    },
};

use crate::{systems::common::components::EntityType, systems::ItemCollider};

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
