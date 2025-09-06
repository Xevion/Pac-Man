use glam::Vec2;
use pacman::constants::{CELL_SIZE, RAW_BOARD};
use pacman::map::builder::Map;
use speculoos::prelude::*;

#[test]
fn test_map_creation() {
    let map = Map::new(RAW_BOARD).unwrap();

    assert_that(&map.graph.nodes().count()).is_greater_than(0);
    assert_that(&map.grid_to_node.is_empty()).is_false();

    // Check that some connections were made
    let mut has_connections = false;
    for intersection in &map.graph.adjacency_list {
        if intersection.edges().next().is_some() {
            has_connections = true;
            break;
        }
    }
    assert_that(&has_connections).is_true();
}

#[test]
fn test_map_node_positions() {
    let map = Map::new(RAW_BOARD).unwrap();

    for (grid_pos, &node_id) in &map.grid_to_node {
        let node = map.graph.get_node(node_id).unwrap();
        let expected_pos = Vec2::new(
            (grid_pos.x as i32 * CELL_SIZE as i32) as f32,
            (grid_pos.y as i32 * CELL_SIZE as i32) as f32,
        ) + Vec2::splat(CELL_SIZE as f32 / 2.0);

        assert_that(&node.position).is_equal_to(expected_pos);
    }
}
