//! Sprite animation construction for player and ghost entities.

use std::collections::HashMap;

use crate::constants::animation;
use crate::error::GameResult;
use crate::map::direction::Direction;
use crate::systems::animation::{DirectionalAnimation, LinearAnimation};
use crate::systems::ghost::{GhostAnimations, GhostType};
use crate::texture::animated::{DirectionalTiles, TileSequence};
use crate::texture::sprite::{AtlasTile, SpriteAtlas};
use crate::texture::sprites::{FrightenedColor, GameSprite, GhostSprite, PacmanSprite};

pub(super) fn create_player_animations(atlas: &SpriteAtlas) -> GameResult<(DirectionalAnimation, AtlasTile)> {
    let up_moving_tiles = [
        SpriteAtlas::get_tile(atlas, &GameSprite::Pacman(PacmanSprite::Moving(Direction::Up, 0)).to_path())?,
        SpriteAtlas::get_tile(atlas, &GameSprite::Pacman(PacmanSprite::Moving(Direction::Up, 1)).to_path())?,
        SpriteAtlas::get_tile(atlas, &GameSprite::Pacman(PacmanSprite::Full).to_path())?,
    ];
    let down_moving_tiles = [
        SpriteAtlas::get_tile(atlas, &GameSprite::Pacman(PacmanSprite::Moving(Direction::Down, 0)).to_path())?,
        SpriteAtlas::get_tile(atlas, &GameSprite::Pacman(PacmanSprite::Moving(Direction::Down, 1)).to_path())?,
        SpriteAtlas::get_tile(atlas, &GameSprite::Pacman(PacmanSprite::Full).to_path())?,
    ];
    let left_moving_tiles = [
        SpriteAtlas::get_tile(atlas, &GameSprite::Pacman(PacmanSprite::Moving(Direction::Left, 0)).to_path())?,
        SpriteAtlas::get_tile(atlas, &GameSprite::Pacman(PacmanSprite::Moving(Direction::Left, 1)).to_path())?,
        SpriteAtlas::get_tile(atlas, &GameSprite::Pacman(PacmanSprite::Full).to_path())?,
    ];
    let right_moving_tiles = [
        SpriteAtlas::get_tile(
            atlas,
            &GameSprite::Pacman(PacmanSprite::Moving(Direction::Right, 0)).to_path(),
        )?,
        SpriteAtlas::get_tile(
            atlas,
            &GameSprite::Pacman(PacmanSprite::Moving(Direction::Right, 1)).to_path(),
        )?,
        SpriteAtlas::get_tile(atlas, &GameSprite::Pacman(PacmanSprite::Full).to_path())?,
    ];

    let moving_tiles = DirectionalTiles::new(
        TileSequence::new(&up_moving_tiles),
        TileSequence::new(&down_moving_tiles),
        TileSequence::new(&left_moving_tiles),
        TileSequence::new(&right_moving_tiles),
    );

    let up_stopped_tile = SpriteAtlas::get_tile(atlas, &GameSprite::Pacman(PacmanSprite::Moving(Direction::Up, 1)).to_path())?;
    let down_stopped_tile =
        SpriteAtlas::get_tile(atlas, &GameSprite::Pacman(PacmanSprite::Moving(Direction::Down, 1)).to_path())?;
    let left_stopped_tile =
        SpriteAtlas::get_tile(atlas, &GameSprite::Pacman(PacmanSprite::Moving(Direction::Left, 1)).to_path())?;
    let right_stopped_tile = SpriteAtlas::get_tile(
        atlas,
        &GameSprite::Pacman(PacmanSprite::Moving(Direction::Right, 1)).to_path(),
    )?;

    let stopped_tiles = DirectionalTiles::new(
        TileSequence::new(&[up_stopped_tile]),
        TileSequence::new(&[down_stopped_tile]),
        TileSequence::new(&[left_stopped_tile]),
        TileSequence::new(&[right_stopped_tile]),
    );

    let player_animation = DirectionalAnimation::new(moving_tiles, stopped_tiles, 5);
    let player_start_sprite = SpriteAtlas::get_tile(atlas, &GameSprite::Pacman(PacmanSprite::Full).to_path())?;

    Ok((player_animation, player_start_sprite))
}

pub(super) fn create_death_animation(atlas: &SpriteAtlas) -> GameResult<LinearAnimation> {
    let mut death_tiles = Vec::new();
    for i in 0..=10 {
        let tile = atlas.get_tile(&GameSprite::Pacman(PacmanSprite::Dying(i)).to_path())?;
        death_tiles.push(tile);
    }

    let tile_sequence = TileSequence::new(&death_tiles);
    Ok(LinearAnimation::new(tile_sequence, 8))
}

pub(super) fn create_ghost_animations(atlas: &SpriteAtlas) -> GameResult<GhostAnimations> {
    // Eaten (eyes) animations - single tile per direction
    let up_eye = atlas.get_tile(&GameSprite::Ghost(GhostSprite::Eyes(Direction::Up)).to_path())?;
    let down_eye = atlas.get_tile(&GameSprite::Ghost(GhostSprite::Eyes(Direction::Down)).to_path())?;
    let left_eye = atlas.get_tile(&GameSprite::Ghost(GhostSprite::Eyes(Direction::Left)).to_path())?;
    let right_eye = atlas.get_tile(&GameSprite::Ghost(GhostSprite::Eyes(Direction::Right)).to_path())?;

    let eyes_tiles = DirectionalTiles::new(
        TileSequence::new(&[up_eye]),
        TileSequence::new(&[down_eye]),
        TileSequence::new(&[left_eye]),
        TileSequence::new(&[right_eye]),
    );
    let eyes = DirectionalAnimation::new(eyes_tiles.clone(), eyes_tiles, animation::GHOST_EATEN_SPEED);

    let mut animations = HashMap::new();

    for ghost_type in [GhostType::Blinky, GhostType::Pinky, GhostType::Inky, GhostType::Clyde] {
        // Normal animations - create directional tiles for each direction
        let up_tiles = [
            atlas.get_tile(&GameSprite::Ghost(GhostSprite::Normal(ghost_type, Direction::Up, 0)).to_path())?,
            atlas.get_tile(&GameSprite::Ghost(GhostSprite::Normal(ghost_type, Direction::Up, 1)).to_path())?,
        ];
        let down_tiles = [
            atlas.get_tile(&GameSprite::Ghost(GhostSprite::Normal(ghost_type, Direction::Down, 0)).to_path())?,
            atlas.get_tile(&GameSprite::Ghost(GhostSprite::Normal(ghost_type, Direction::Down, 1)).to_path())?,
        ];
        let left_tiles = [
            atlas.get_tile(&GameSprite::Ghost(GhostSprite::Normal(ghost_type, Direction::Left, 0)).to_path())?,
            atlas.get_tile(&GameSprite::Ghost(GhostSprite::Normal(ghost_type, Direction::Left, 1)).to_path())?,
        ];
        let right_tiles = [
            atlas.get_tile(&GameSprite::Ghost(GhostSprite::Normal(ghost_type, Direction::Right, 0)).to_path())?,
            atlas.get_tile(&GameSprite::Ghost(GhostSprite::Normal(ghost_type, Direction::Right, 1)).to_path())?,
        ];

        let normal_moving = DirectionalTiles::new(
            TileSequence::new(&up_tiles),
            TileSequence::new(&down_tiles),
            TileSequence::new(&left_tiles),
            TileSequence::new(&right_tiles),
        );
        let normal = DirectionalAnimation::new(normal_moving.clone(), normal_moving, animation::GHOST_NORMAL_SPEED);

        animations.insert(ghost_type, normal);
    }

    let (frightened, frightened_flashing) = {
        // Load frightened animation tiles (same for all ghosts)
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
