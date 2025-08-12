use glam::Vec2;
use pacman::constants::{CELL_SIZE, RAW_BOARD};
use pacman::map::Map;
use sdl2::render::Texture;

#[test]
fn test_map_creation() {
    let map = Map::new(RAW_BOARD).unwrap();

    assert!(map.graph.node_count() > 0);
    assert!(!map.grid_to_node.is_empty());

    // Check that some connections were made
    let mut has_connections = false;
    for intersection in &map.graph.adjacency_list {
        if intersection.edges().next().is_some() {
            has_connections = true;
            break;
        }
    }
    assert!(has_connections);
}

#[test]
fn test_map_node_positions() {
    let map = Map::new(RAW_BOARD).unwrap();

    for (grid_pos, &node_id) in &map.grid_to_node {
        let node = map.graph.get_node(node_id).unwrap();
        let expected_pos = Vec2::new((grid_pos.x * CELL_SIZE as i32) as f32, (grid_pos.y * CELL_SIZE as i32) as f32)
            + Vec2::splat(CELL_SIZE as f32 / 2.0);

        assert_eq!(node.position, expected_pos);
    }
}

#[test]
fn test_generate_items() {
    use pacman::texture::sprite::{AtlasMapper, MapperFrame, SpriteAtlas};
    use std::collections::HashMap;

    let map = Map::new(RAW_BOARD).unwrap();

    // Create a minimal atlas for testing
    let mut frames = HashMap::new();
    frames.insert(
        "maze/pellet.png".to_string(),
        MapperFrame {
            x: 0,
            y: 0,
            width: 8,
            height: 8,
        },
    );
    frames.insert(
        "maze/energizer.png".to_string(),
        MapperFrame {
            x: 8,
            y: 0,
            width: 8,
            height: 8,
        },
    );

    let mapper = AtlasMapper { frames };
    let texture = unsafe { std::mem::transmute::<usize, Texture<'static>>(0usize) };
    let atlas = SpriteAtlas::new(texture, mapper);

    let items = map.generate_items(&atlas).unwrap();

    // Verify we have items
    assert!(!items.is_empty());

    // Count different types
    let pellet_count = items
        .iter()
        .filter(|item| matches!(item.item_type, pacman::entity::item::ItemType::Pellet))
        .count();
    let energizer_count = items
        .iter()
        .filter(|item| matches!(item.item_type, pacman::entity::item::ItemType::Energizer))
        .count();

    // Should have both types
    assert_eq!(pellet_count, 240);
    assert_eq!(energizer_count, 4);

    // All items should be uncollected initially
    assert!(items.iter().all(|item| !item.is_collected()));

    // All items should have valid node indices
    assert!(items.iter().all(|item| item.node_index < map.graph.node_count()));
}
