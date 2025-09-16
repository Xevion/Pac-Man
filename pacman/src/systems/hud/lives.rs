use std::cmp::Ordering;

use crate::constants::{BOARD_BOTTOM_PIXEL_OFFSET, CANVAS_SIZE, CELL_SIZE};
use crate::error::GameError;
use crate::map::direction::Direction;
use crate::systems::{PixelPosition, PlayerLife, PlayerLives, Renderable};
use crate::texture::sprite::SpriteAtlas;
use crate::texture::sprites::{GameSprite, PacmanSprite};
use bevy_ecs::entity::Entity;
use bevy_ecs::event::EventWriter;
use bevy_ecs::system::{Commands, NonSendMut, Query, Res};
use glam::Vec2;

/// Calculates the pixel position for a life sprite based on its index
fn calculate_life_sprite_position(index: u32) -> Vec2 {
    let start_x = CELL_SIZE * 2; // 2 cells from left
    let start_y = CANVAS_SIZE.y - BOARD_BOTTOM_PIXEL_OFFSET.y + (CELL_SIZE / 2) + 1; // In bottom area
    let sprite_spacing = CELL_SIZE + CELL_SIZE / 2; // 1.5 cells between sprites

    let x = start_x + ((index as f32) * (sprite_spacing as f32 * 1.5)).round() as u32;
    let y = start_y - CELL_SIZE / 2;

    Vec2::new((x + CELL_SIZE) as f32, (y + CELL_SIZE) as f32)
}

/// System that manages player life sprite entities.
/// Spawns and despawns life sprite entities based on changes to PlayerLives resource.
/// Each life sprite is positioned based on its index (0, 1, 2, etc. from left to right).
pub fn player_life_sprite_system(
    mut commands: Commands,
    atlas: NonSendMut<SpriteAtlas>,
    current_life_sprites: Query<(Entity, &PlayerLife)>,
    player_lives: Res<PlayerLives>,
    mut errors: EventWriter<GameError>,
) {
    let displayed_lives = player_lives.0.saturating_sub(1);

    // Get current life sprite entities, sorted by index
    let mut current_sprites: Vec<_> = current_life_sprites.iter().collect();
    current_sprites.sort_by_key(|(_, life)| life.index);
    let current_count = current_sprites.len() as u8;

    // Calculate the difference
    let diff = (displayed_lives as i8) - (current_count as i8);

    match diff.cmp(&0) {
        // Ignore when the number of lives displayed is correct
        Ordering::Equal => {}
        // Spawn new life sprites
        Ordering::Greater => {
            let life_sprite = match atlas.get_tile(&GameSprite::Pacman(PacmanSprite::Moving(Direction::Left, 1)).to_path()) {
                Ok(sprite) => sprite,
                Err(e) => {
                    errors.write(e.into());
                    return;
                }
            };

            for i in 0..diff {
                let position = calculate_life_sprite_position(i as u32);

                commands.spawn((
                    PlayerLife { index: i as u32 },
                    Renderable {
                        sprite: life_sprite,
                        layer: 255, // High layer to render on top
                    },
                    PixelPosition {
                        pixel_position: position,
                    },
                ));
            }
        }
        // Remove excess life sprites (highest indices first)
        Ordering::Less => {
            let to_remove = diff.unsigned_abs();
            let sprites_to_remove: Vec<_> = current_sprites
                .iter()
                .rev() // Start from highest index
                .take(to_remove as usize)
                .map(|(entity, _)| *entity)
                .collect();

            for entity in sprites_to_remove {
                commands.entity(entity).despawn();
            }
        }
    }
}
