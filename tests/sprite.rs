use glam::U16Vec2;
use pacman::texture::sprite::{AtlasMapper, AtlasTile, MapperFrame};
use sdl2::pixels::Color;
use std::collections::HashMap;

mod common;

#[test]
fn test_atlas_mapper_frame_lookup() {
    let mut frames = HashMap::new();
    frames.insert(
        "test".to_string(),
        MapperFrame {
            pos: U16Vec2::new(10, 20),
            size: U16Vec2::new(32, 64),
        },
    );

    let mapper = AtlasMapper { frames };

    // Test direct frame lookup
    let frame = mapper.frames.get("test");
    assert!(frame.is_some());
    let frame = frame.unwrap();
    assert_eq!(frame.pos, U16Vec2::new(10, 20));
    assert_eq!(frame.size, U16Vec2::new(32, 64));
}

#[test]
fn test_atlas_mapper_multiple_frames() {
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

    assert_eq!(mapper.frames.len(), 2);
    assert!(mapper.frames.contains_key("tile1"));
    assert!(mapper.frames.contains_key("tile2"));
    assert!(!mapper.frames.contains_key("tile3"));
    assert!(!mapper.frames.contains_key("nonexistent"));
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
