use pacman::map::direction::Direction;
use pacman::map::graph::{Graph, Node, TraversalFlags};
use speculoos::prelude::*;

mod common;

#[test]
fn test_graph_basic_operations() {
    let mut graph = Graph::new();
    let node1 = graph.add_node(Node {
        position: glam::Vec2::new(0.0, 0.0),
    });
    let node2 = graph.add_node(Node {
        position: glam::Vec2::new(16.0, 0.0),
    });

    assert_that(&graph.nodes().count()).is_equal_to(2);
    assert_that(&graph.get_node(node1).is_some()).is_true();
    assert_that(&graph.get_node(node2).is_some()).is_true();
    assert_that(&graph.get_node(999).is_none()).is_true();
}

#[test]
fn test_graph_connect() {
    let mut graph = Graph::new();
    let node1 = graph.add_node(Node {
        position: glam::Vec2::new(0.0, 0.0),
    });
    let node2 = graph.add_node(Node {
        position: glam::Vec2::new(16.0, 0.0),
    });

    assert_that(&graph.connect(node1, node2, false, None, Direction::Right).is_ok()).is_true();

    let edge1 = graph.find_edge_in_direction(node1, Direction::Right);
    let edge2 = graph.find_edge_in_direction(node2, Direction::Left);

    assert_that(&edge1.is_some()).is_true();
    assert_that(&edge2.is_some()).is_true();
    assert_that(&edge1.unwrap().target).is_equal_to(node2);
    assert_that(&edge2.unwrap().target).is_equal_to(node1);
}

#[test]
fn test_graph_connect_errors() {
    let mut graph = Graph::new();
    let node1 = graph.add_node(Node {
        position: glam::Vec2::new(0.0, 0.0),
    });

    assert_that(&graph.connect(node1, 999, false, None, Direction::Right).is_err()).is_true();
    assert_that(&graph.connect(999, node1, false, None, Direction::Right).is_err()).is_true();
}

#[test]
fn test_graph_edge_permissions() {
    let mut graph = Graph::new();
    let node1 = graph.add_node(Node {
        position: glam::Vec2::new(0.0, 0.0),
    });
    let node2 = graph.add_node(Node {
        position: glam::Vec2::new(16.0, 0.0),
    });

    graph
        .add_edge(node1, node2, false, None, Direction::Right, TraversalFlags::GHOST)
        .unwrap();

    let edge = graph.find_edge_in_direction(node1, Direction::Right).unwrap();
    assert_that(&edge.traversal_flags).is_equal_to(TraversalFlags::GHOST);
}

#[test]
fn should_add_connected_node() {
    let mut graph = Graph::new();
    let node1 = graph.add_node(Node {
        position: glam::Vec2::new(0.0, 0.0),
    });

    let node2 = graph
        .add_connected(
            node1,
            Direction::Right,
            Node {
                position: glam::Vec2::new(16.0, 0.0),
            },
        )
        .unwrap();

    assert_that(&graph.nodes().count()).is_equal_to(2);
    let edge = graph.find_edge(node1, node2);
    assert_that(&edge.is_some()).is_true();
    assert_that(&edge.unwrap().direction).is_equal_to(Direction::Right);
}

#[test]
fn should_error_on_negative_edge_distance() {
    let mut graph = Graph::new();
    let node1 = graph.add_node(Node {
        position: glam::Vec2::new(0.0, 0.0),
    });
    let node2 = graph.add_node(Node {
        position: glam::Vec2::new(16.0, 0.0),
    });

    let result = graph.add_edge(node1, node2, false, Some(-1.0), Direction::Right, TraversalFlags::ALL);
    assert_that(&result.is_err()).is_true();
}

#[test]
fn should_error_on_duplicate_edge_without_replace() {
    let mut graph = common::create_test_graph();
    let result = graph.add_edge(0, 1, false, None, Direction::Right, TraversalFlags::ALL);
    assert_that(&result.is_err()).is_true();
}

#[test]
fn should_allow_replacing_an_edge() {
    let mut graph = common::create_test_graph();
    let result = graph.add_edge(0, 1, true, Some(42.0), Direction::Right, TraversalFlags::ALL);
    assert_that(&result.is_ok()).is_true();

    let edge = graph.find_edge(0, 1).unwrap();
    assert_that(&edge.distance).is_equal_to(42.0);
}

#[test]
fn should_find_edge_between_nodes() {
    let graph = common::create_test_graph();
    let edge = graph.find_edge(0, 1);
    assert_that(&edge.is_some()).is_true();
    assert_that(&edge.unwrap().target).is_equal_to(1);

    let non_existent_edge = graph.find_edge(0, 99);
    assert_that(&non_existent_edge.is_none()).is_true();
}
