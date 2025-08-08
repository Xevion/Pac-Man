use glam::U16Vec2;
use pacman::texture::blinking::BlinkingTexture;
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
fn test_blinking_texture() {
    let tile = mock_atlas_tile(1);
    let mut texture = BlinkingTexture::new(tile, 0.5);

    assert_eq!(texture.is_on(), true);

    texture.tick(0.5);
    assert_eq!(texture.is_on(), false);

    texture.tick(0.5);
    assert_eq!(texture.is_on(), true);

    texture.tick(0.5);
    assert_eq!(texture.is_on(), false);
}

#[test]
fn test_blinking_texture_partial_duration() {
    let tile = mock_atlas_tile(1);
    let mut texture = BlinkingTexture::new(tile, 0.5);

    texture.tick(0.625);
    assert_eq!(texture.is_on(), false);
    assert_eq!(texture.time_bank(), 0.125);
}

#[test]
fn test_blinking_texture_negative_time() {
    let tile = mock_atlas_tile(1);
    let mut texture = BlinkingTexture::new(tile, 0.5);

    texture.tick(-0.1);
    assert_eq!(texture.is_on(), true);
    assert_eq!(texture.time_bank(), -0.1);
}
