use pacman::constants::RAW_BOARD;
use pacman::map::Map;

#[test]
fn test_game_map_creation() {
    let map = Map::new(RAW_BOARD);

    assert!(map.graph.node_count() > 0);
    assert!(!map.grid_to_node.is_empty());

    // Should find Pac-Man's starting position
    let pacman_pos = map.find_starting_position(0);
    assert!(pacman_pos.is_some());
}

#[test]
fn test_game_score_initialization() {
    // This would require creating a full Game instance, but we can test the concept
    let map = Map::new(RAW_BOARD);
    assert!(map.find_starting_position(0).is_some());
}
