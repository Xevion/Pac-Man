use pacman::constants::RAW_BOARD;
use pacman::map::Map;

mod collision;
mod item;

#[test]
fn test_game_map_creation() {
    let map = Map::new(RAW_BOARD).unwrap();

    assert!(map.graph.node_count() > 0);
    assert!(!map.grid_to_node.is_empty());
}
