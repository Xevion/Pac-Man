//! Sprite animation construction for player and ghost entities.

use std::collections::HashMap;

use crate::constants::animation;
use crate::error::{GameResult, TextureError};
use crate::map::direction::Direction;
use crate::systems::animation::{DirectionalAnimation, LinearAnimation};
use crate::systems::ghost::{GhostAnimations, GhostType};
use crate::texture::animated::{DirectionalTiles, TileSequence};
use crate::texture::sprite::{AtlasTile, SpriteAtlas};
use crate::texture::sprites::{FrightenedColor, GameSprite, GhostSprite, PacmanSprite};

/// Loads directional tiles from the atlas using a sprite-generating closure.
fn load_directional_tiles(
    atlas: &SpriteAtlas,
    frames_per_direction: usize,
    sprite_fn: impl Fn(Direction, usize) -> GameSprite,
) -> GameResult<DirectionalTiles> {
    let load_dir = |dir: Direction| -> GameResult<TileSequence> {
        let tiles: Vec<AtlasTile> = (0..frames_per_direction)
            .map(|i| atlas.get_tile(&sprite_fn(dir, i).to_path()))
            .collect::<Result<_, TextureError>>()?;
        Ok(TileSequence::new(&tiles))
    };
    Ok(DirectionalTiles::new(
        load_dir(Direction::Up)?,
        load_dir(Direction::Down)?,
        load_dir(Direction::Left)?,
        load_dir(Direction::Right)?,
    ))
}

pub(super) fn create_player_animations(atlas: &SpriteAtlas) -> GameResult<(DirectionalAnimation, AtlasTile)> {
    let full_tile = atlas.get_tile(&GameSprite::Pacman(PacmanSprite::Full).to_path())?;

    let moving_tiles = load_directional_tiles(atlas, 3, |dir, i| {
        if i < 2 {
            GameSprite::Pacman(PacmanSprite::Moving(dir, i as u8))
        } else {
            GameSprite::Pacman(PacmanSprite::Full)
        }
    })?;

    let stopped_tiles = load_directional_tiles(atlas, 1, |dir, _| GameSprite::Pacman(PacmanSprite::Moving(dir, 1)))?;

    let player_animation = DirectionalAnimation::new(moving_tiles, stopped_tiles, 5);
    Ok((player_animation, full_tile))
}

pub(super) fn create_death_animation(atlas: &SpriteAtlas) -> GameResult<LinearAnimation> {
    let death_tiles: Vec<AtlasTile> = (0..=10)
        .map(|i| atlas.get_tile(&GameSprite::Pacman(PacmanSprite::Dying(i)).to_path()))
        .collect::<Result<_, TextureError>>()?;

    Ok(LinearAnimation::new(TileSequence::new(&death_tiles), 8))
}

pub(super) fn create_ghost_animations(atlas: &SpriteAtlas) -> GameResult<GhostAnimations> {
    let eyes_tiles = load_directional_tiles(atlas, 1, |dir, _| GameSprite::Ghost(GhostSprite::Eyes(dir)))?;
    let eyes = DirectionalAnimation::new(eyes_tiles.clone(), eyes_tiles, animation::GHOST_EATEN_SPEED);

    let mut animations = HashMap::new();

    for ghost_type in [GhostType::Blinky, GhostType::Pinky, GhostType::Inky, GhostType::Clyde] {
        let normal_moving = load_directional_tiles(atlas, 2, |dir, i| {
            GameSprite::Ghost(GhostSprite::Normal(ghost_type, dir, i as u8))
        })?;
        let normal = DirectionalAnimation::new(normal_moving.clone(), normal_moving, animation::GHOST_NORMAL_SPEED);
        animations.insert(ghost_type, normal);
    }

    let (frightened, frightened_flashing) = {
        let frightened_blue_a =
            atlas.get_tile(&GameSprite::Ghost(GhostSprite::Frightened(FrightenedColor::Blue, 0)).to_path())?;
        let frightened_blue_b =
            atlas.get_tile(&GameSprite::Ghost(GhostSprite::Frightened(FrightenedColor::Blue, 1)).to_path())?;
        let frightened_white_a =
            atlas.get_tile(&GameSprite::Ghost(GhostSprite::Frightened(FrightenedColor::White, 0)).to_path())?;
        let frightened_white_b =
            atlas.get_tile(&GameSprite::Ghost(GhostSprite::Frightened(FrightenedColor::White, 1)).to_path())?;

        (
            LinearAnimation::new(
                TileSequence::new(&[frightened_blue_a, frightened_blue_b]),
                animation::GHOST_NORMAL_SPEED,
            ),
            LinearAnimation::new(
                TileSequence::new(&[frightened_blue_a, frightened_white_a, frightened_blue_b, frightened_white_b]),
                animation::GHOST_FRIGHTENED_SPEED,
            ),
        )
    };

    Ok(GhostAnimations::new(animations, eyes, frightened, frightened_flashing))
}
