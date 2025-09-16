use glam::Vec2;
use pacman::constants::{CELL_SIZE, RAW_BOARD};
use pacman::map::builder::Map;
use pacman::map::graph::TraversalFlags;
use speculoos::prelude::*;

#[test]
fn test_map_creation_success() {
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
fn test_map_node_positions_accuracy() {
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

#[test]
fn test_start_positions_are_valid() {
    let map = Map::new(RAW_BOARD).unwrap();
    let positions = &map.start_positions;

    // All start positions should exist in the graph
    assert_that(&map.graph.get_node(positions.pacman)).is_some();
    assert_that(&map.graph.get_node(positions.blinky)).is_some();
    assert_that(&map.graph.get_node(positions.pinky)).is_some();
    assert_that(&map.graph.get_node(positions.inky)).is_some();
    assert_that(&map.graph.get_node(positions.clyde)).is_some();
}

#[test]
fn test_ghost_house_has_ghost_only_entrance() {
    let map = Map::new(RAW_BOARD).unwrap();

    // Find the house entrance node
    let house_entrance = map.start_positions.blinky;

    // Check that there's a ghost-only connection from the house entrance
    let mut has_ghost_only_connection = false;
    for edge in map.graph.adjacency_list[house_entrance as usize].edges() {
        if edge.traversal_flags == TraversalFlags::GHOST {
            has_ghost_only_connection = true;
            break;
        }
    }
    assert_that(&has_ghost_only_connection).is_true();
}

#[test]
fn test_tunnel_connections_exist() {
    let map = Map::new(RAW_BOARD).unwrap();

    // Find tunnel nodes by looking for nodes with zero-distance connections
    let mut has_tunnel_connection = false;
    for intersection in &map.graph.adjacency_list {
        for edge in intersection.edges() {
            if edge.distance == 0.0f32 {
                has_tunnel_connection = true;
                break;
            }
        }
        if has_tunnel_connection {
            break;
        }
    }
    assert_that(&has_tunnel_connection).is_true();
}
