use glam::U16Vec2;
use pacman::texture::sprite::{AtlasMapper, AtlasTile, MapperFrame, SpriteAtlas};
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
            pos: U16Vec2::new(10, 20),
            size: U16Vec2::new(32, 64),
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
            pos: U16Vec2::new(0, 0),
            size: U16Vec2::new(32, 32),
        },
    );
    frames.insert(
        "tile2".to_string(),
        MapperFrame {
            pos: U16Vec2::new(32, 0),
            size: U16Vec2::new(64, 64),
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

#[test]
fn test_atlas_tile_new_and_with_color() {
    let pos = U16Vec2::new(10, 20);
    let size = U16Vec2::new(30, 40);
    let color = Color::RGB(100, 150, 200);

    let tile = AtlasTile::new(pos, size, None);
    assert_eq!(tile.pos, pos);
    assert_eq!(tile.size, size);
    assert_eq!(tile.color, None);

    let tile_with_color = tile.with_color(color);
    assert_eq!(tile_with_color.color, Some(color));
}
