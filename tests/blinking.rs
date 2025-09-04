use pacman::texture::blinking::BlinkingTexture;

mod common;

#[test]
fn test_blinking_texture() {
    let tile = common::mock_atlas_tile(1);
    let mut texture = BlinkingTexture::new(tile, 0.5);

    assert!(texture.is_on());

    texture.tick(0.5);
    assert!(!texture.is_on());

    texture.tick(0.5);
    assert!(texture.is_on());

    texture.tick(0.5);
    assert!(!texture.is_on());
}

#[test]
fn test_blinking_texture_partial_duration() {
    let tile = common::mock_atlas_tile(1);
    let mut texture = BlinkingTexture::new(tile, 0.5);

    texture.tick(0.625);
    assert!(!texture.is_on());
    assert_eq!(texture.time_bank(), 0.125);
}

#[test]
fn test_blinking_texture_negative_time() {
    let tile = common::mock_atlas_tile(1);
    let mut texture = BlinkingTexture::new(tile, 0.5);

    texture.tick(-0.1);
    assert!(texture.is_on());
    assert_eq!(texture.time_bank(), -0.1);
}
