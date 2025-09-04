use glam::Vec2;
use pacman::constants::{CELL_SIZE, RAW_BOARD};
use pacman::map::builder::Map;

#[test]
fn test_map_creation() {
    let map = Map::new(RAW_BOARD).unwrap();

    assert!(map.graph.nodes().count() > 0);
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
        let expected_pos = Vec2::new(
            (grid_pos.x as i32 * CELL_SIZE as i32) as f32,
            (grid_pos.y as i32 * CELL_SIZE as i32) as f32,
        ) + Vec2::splat(CELL_SIZE as f32 / 2.0);

        assert_eq!(node.position, expected_pos);
    }
}
