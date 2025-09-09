//! Tests for the sprite path generation.
use pacman::{
    game::ATLAS_FRAMES,
    map::direction::Direction,
    systems::Ghost,
    texture::sprites::{FrightenedColor, GameSprite, GhostSprite, MazeSprite, PacmanSprite},
};

#[test]
fn test_all_sprite_paths_exist() {
    let mut sprites_to_test = Vec::new();

    // Pac-Man sprites
    for &dir in &[Direction::Up, Direction::Down, Direction::Left, Direction::Right] {
        for frame in 0..2 {
            sprites_to_test.push(GameSprite::Pacman(PacmanSprite::Moving(dir, frame)));
        }
    }
    sprites_to_test.push(GameSprite::Pacman(PacmanSprite::Full));
    for frame in 0..=10 {
        sprites_to_test.push(GameSprite::Pacman(PacmanSprite::Dying(frame)));
    }

    // Ghost sprites
    for &ghost in &[Ghost::Blinky, Ghost::Pinky, Ghost::Inky, Ghost::Clyde] {
        for &dir in &[Direction::Up, Direction::Down, Direction::Left, Direction::Right] {
            for frame in 0..2 {
                sprites_to_test.push(GameSprite::Ghost(GhostSprite::Normal(ghost, dir, frame)));
            }
            sprites_to_test.push(GameSprite::Ghost(GhostSprite::Eyes(dir)));
        }
    }
    for &color in &[FrightenedColor::Blue, FrightenedColor::White] {
        for frame in 0..2 {
            sprites_to_test.push(GameSprite::Ghost(GhostSprite::Frightened(color, frame)));
        }
    }

    // Maze sprites
    for i in 0..=34 {
        sprites_to_test.push(GameSprite::Maze(MazeSprite::Tile(i)));
    }
    sprites_to_test.push(GameSprite::Maze(MazeSprite::Pellet));
    sprites_to_test.push(GameSprite::Maze(MazeSprite::Energizer));

    for sprite in sprites_to_test {
        let path = sprite.to_path();
        assert!(
            ATLAS_FRAMES.contains_key(&path),
            "Sprite path '{}' does not exist in the atlas.",
            path
        );
    }
}

#[test]
fn test_invalid_sprite_paths_do_not_exist() {
    let invalid_sprites = vec![
        // An invalid Pac-Man dying frame
        GameSprite::Pacman(PacmanSprite::Dying(99)),
        // An invalid maze tile
        GameSprite::Maze(MazeSprite::Tile(99)),
    ];

    for sprite in invalid_sprites {
        let path = sprite.to_path();
        assert!(
            !ATLAS_FRAMES.contains_key(&path),
            "Invalid sprite path '{}' was found in the atlas, but it should not exist.",
            path
        );
    }
}
