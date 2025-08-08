use glam::U16Vec2;
use pacman::entity::direction::Direction;
use pacman::texture::animated::AnimatedTexture;
use pacman::texture::directional::DirectionalAnimatedTexture;
use pacman::texture::sprite::AtlasTile;
use sdl2::pixels::Color;
use std::collections::HashMap;

fn mock_atlas_tile(id: u32) -> AtlasTile {
    AtlasTile {
        pos: U16Vec2::new(0, 0),
        size: U16Vec2::new(16, 16),
        color: Some(Color::RGB(id as u8, 0, 0)),
    }
}

fn mock_animated_texture(id: u32) -> AnimatedTexture {
    AnimatedTexture::new(vec![mock_atlas_tile(id)], 0.1).expect("Invalid frame duration")
}

#[test]
fn test_directional_texture_partial_directions() {
    let mut textures = HashMap::new();
    textures.insert(Direction::Up, mock_animated_texture(1));

    let texture = DirectionalAnimatedTexture::new(textures, HashMap::new());

    assert_eq!(texture.texture_count(), 1);
    assert!(texture.has_direction(Direction::Up));
    assert!(!texture.has_direction(Direction::Down));
    assert!(!texture.has_direction(Direction::Left));
    assert!(!texture.has_direction(Direction::Right));
}

#[test]
fn test_directional_texture_all_directions() {
    let mut textures = HashMap::new();
    let directions = [
        (Direction::Up, 1),
        (Direction::Down, 2),
        (Direction::Left, 3),
        (Direction::Right, 4),
    ];

    for (direction, id) in directions {
        textures.insert(direction, mock_animated_texture(id));
    }

    let texture = DirectionalAnimatedTexture::new(textures, HashMap::new());

    assert_eq!(texture.texture_count(), 4);
    for direction in &[Direction::Up, Direction::Down, Direction::Left, Direction::Right] {
        assert!(texture.has_direction(*direction));
    }
}
