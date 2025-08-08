use pacman::texture::sprite::{AtlasMapper, MapperFrame, SpriteAtlas};
use sdl2::pixels::Color;
use std::collections::HashMap;

fn mock_texture() -> sdl2::render::Texture<'static> {
    unsafe { std::mem::transmute(0usize) }
}

#[test]
fn test_sprite_atlas_basic() {
    let mut frames = HashMap::new();
    frames.insert(
        "test".to_string(),
        MapperFrame {
            x: 10,
            y: 20,
            width: 32,
            height: 64,
        },
    );

    let mapper = AtlasMapper { frames };
    let texture = mock_texture();
    let atlas = SpriteAtlas::new(texture, mapper);

    let tile = atlas.get_tile("test");
    assert!(tile.is_some());
    let tile = tile.unwrap();
    assert_eq!(tile.pos, glam::U16Vec2::new(10, 20));
    assert_eq!(tile.size, glam::U16Vec2::new(32, 64));
    assert_eq!(tile.color, None);
}

#[test]
fn test_sprite_atlas_multiple_tiles() {
    let mut frames = HashMap::new();
    frames.insert(
        "tile1".to_string(),
        MapperFrame {
            x: 0,
            y: 0,
            width: 32,
            height: 32,
        },
    );
    frames.insert(
        "tile2".to_string(),
        MapperFrame {
            x: 32,
            y: 0,
            width: 64,
            height: 64,
        },
    );

    let mapper = AtlasMapper { frames };
    let texture = mock_texture();
    let atlas = SpriteAtlas::new(texture, mapper);

    assert_eq!(atlas.tiles_count(), 2);
    assert!(atlas.has_tile("tile1"));
    assert!(atlas.has_tile("tile2"));
    assert!(!atlas.has_tile("tile3"));
    assert!(atlas.get_tile("nonexistent").is_none());
}

#[test]
fn test_sprite_atlas_color() {
    let mapper = AtlasMapper { frames: HashMap::new() };
    let texture = mock_texture();
    let mut atlas = SpriteAtlas::new(texture, mapper);

    assert_eq!(atlas.default_color(), None);

    let color = Color::RGB(255, 0, 0);
    atlas.set_color(color);
    assert_eq!(atlas.default_color(), Some(color));
}
