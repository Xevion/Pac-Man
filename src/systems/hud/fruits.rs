use crate::systems::item::FruitType;
use crate::texture::sprites::GameSprite;
use bevy_ecs::component::Component;
use bevy_ecs::resource::Resource;

#[derive(Component)]
pub struct FruitInHud {
    pub index: u32,
}

#[derive(Resource, Default)]
pub struct FruitSprites(pub Vec<FruitType>);

use crate::constants::{BOARD_BOTTOM_PIXEL_OFFSET, CANVAS_SIZE, CELL_SIZE};
use crate::error::GameError;
use crate::systems::{PixelPosition, Renderable};
use crate::texture::sprite::SpriteAtlas;
use bevy_ecs::entity::Entity;
use bevy_ecs::event::EventWriter;
use bevy_ecs::system::{Commands, NonSendMut, Query, Res};
use glam::Vec2;

/// Calculates the pixel position for a fruit sprite based on its index
fn calculate_fruit_sprite_position(index: u32) -> Vec2 {
    let start_x = CANVAS_SIZE.x - CELL_SIZE * 2; // 2 cells from right
    let start_y = CANVAS_SIZE.y - BOARD_BOTTOM_PIXEL_OFFSET.y + (CELL_SIZE / 2) + 1; // In bottom area
    let sprite_spacing = CELL_SIZE + CELL_SIZE / 2; // 1.5 cells between sprites

    let x = start_x - ((index as f32) * (sprite_spacing as f32 * 1.5)).round() as u32;
    let y = start_y - (1 + CELL_SIZE / 2);

    Vec2::new((x - CELL_SIZE) as f32, (y + CELL_SIZE) as f32)
}

/// System that manages fruit sprite entities in the HUD.
/// Spawns and despawns fruit sprite entities based on changes to FruitSprites resource.
/// Displays up to 6 fruits, sorted by value.
pub fn fruit_sprite_system(
    mut commands: Commands,
    atlas: NonSendMut<SpriteAtlas>,
    current_fruit_sprites: Query<(Entity, &FruitInHud)>,
    fruit_sprites: Res<FruitSprites>,
    mut errors: EventWriter<GameError>,
) {
    // We only want to display the greatest 6 fruits
    let fruits_to_display: Vec<_> = fruit_sprites.0.iter().rev().take(6).collect();

    let mut current_sprites: Vec<_> = current_fruit_sprites.iter().collect();
    current_sprites.sort_by_key(|(_, fruit)| fruit.index);

    // Despawn all current sprites. We will respawn them.
    // This is simpler than trying to match them up.
    for (entity, _) in &current_sprites {
        commands.entity(*entity).despawn();
    }

    for (i, fruit_type) in fruits_to_display.iter().enumerate() {
        let fruit_sprite = match atlas.get_tile(&GameSprite::Fruit(**fruit_type).to_path()) {
            Ok(sprite) => sprite,
            Err(e) => {
                errors.write(e.into());
                continue;
            }
        };

        let position = calculate_fruit_sprite_position(i as u32);

        commands.spawn((
            FruitInHud { index: i as u32 },
            Renderable {
                sprite: fruit_sprite,
                layer: 255, // High layer to render on top
            },
            PixelPosition {
                pixel_position: position,
            },
        ));
    }
}
