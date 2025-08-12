use glam::U16Vec2;
use pacman::error::{AnimatedTextureError, GameError, TextureError};
use pacman::texture::animated::AnimatedTexture;
use pacman::texture::sprite::AtlasTile;
use sdl2::pixels::Color;

fn mock_atlas_tile(id: u32) -> AtlasTile {
    AtlasTile {
        pos: U16Vec2::new(0, 0),
        size: U16Vec2::new(16, 16),
        color: Some(Color::RGB(id as u8, 0, 0)),
    }
}

#[test]
fn test_animated_texture_creation_errors() {
    let tiles = vec![mock_atlas_tile(1), mock_atlas_tile(2)];

    assert!(matches!(
        AnimatedTexture::new(tiles.clone(), 0.0).unwrap_err(),
        GameError::Texture(TextureError::Animated(AnimatedTextureError::InvalidFrameDuration(0.0)))
    ));

    assert!(matches!(
        AnimatedTexture::new(tiles, -0.1).unwrap_err(),
        GameError::Texture(TextureError::Animated(AnimatedTextureError::InvalidFrameDuration(-0.1)))
    ));
}

#[test]
fn test_animated_texture_advancement() {
    let tiles = vec![mock_atlas_tile(1), mock_atlas_tile(2), mock_atlas_tile(3)];
    let mut texture = AnimatedTexture::new(tiles, 0.1).unwrap();

    assert_eq!(texture.current_frame(), 0);

    texture.tick(0.25);
    assert_eq!(texture.current_frame(), 2);
    assert!((texture.time_bank() - 0.05).abs() < 0.001);
}

#[test]
fn test_animated_texture_wrap_around() {
    let tiles = vec![mock_atlas_tile(1), mock_atlas_tile(2)];
    let mut texture = AnimatedTexture::new(tiles, 0.1).unwrap();

    texture.tick(0.1);
    assert_eq!(texture.current_frame(), 1);

    texture.tick(0.1);
    assert_eq!(texture.current_frame(), 0);
}

#[test]
fn test_animated_texture_single_frame() {
    let tiles = vec![mock_atlas_tile(1)];
    let mut texture = AnimatedTexture::new(tiles, 0.1).unwrap();

    texture.tick(0.1);
    assert_eq!(texture.current_frame(), 0);
    assert_eq!(texture.current_tile().color.unwrap().r, 1);
}
