use pacman::entity::direction::Direction;
use pacman::entity::graph::{Graph, Node, TraversalFlags};

fn create_test_graph() -> Graph {
    let mut graph = Graph::new();
    let node1 = graph.add_node(Node {
        position: glam::Vec2::new(0.0, 0.0),
    });
    let node2 = graph.add_node(Node {
        position: glam::Vec2::new(16.0, 0.0),
    });
    let node3 = graph.add_node(Node {
        position: glam::Vec2::new(0.0, 16.0),
    });

    graph.connect(node1, node2, false, None, Direction::Right).unwrap();
    graph.connect(node1, node3, false, None, Direction::Down).unwrap();

    graph
}

#[test]
fn test_graph_basic_operations() {
    let mut graph = Graph::new();
    let node1 = graph.add_node(Node {
        position: glam::Vec2::new(0.0, 0.0),
    });
    let node2 = graph.add_node(Node {
        position: glam::Vec2::new(16.0, 0.0),
    });

    assert_eq!(graph.node_count(), 2);
    assert!(graph.get_node(node1).is_some());
    assert!(graph.get_node(node2).is_some());
    assert!(graph.get_node(999).is_none());
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

    assert!(graph.connect(node1, node2, false, None, Direction::Right).is_ok());

    let edge1 = graph.find_edge_in_direction(node1, Direction::Right);
    let edge2 = graph.find_edge_in_direction(node2, Direction::Left);

    assert!(edge1.is_some());
    assert!(edge2.is_some());
    assert_eq!(edge1.unwrap().target, node2);
    assert_eq!(edge2.unwrap().target, node1);
}

#[test]
fn test_graph_connect_errors() {
    let mut graph = Graph::new();
    let node1 = graph.add_node(Node {
        position: glam::Vec2::new(0.0, 0.0),
    });

    assert!(graph.connect(node1, 999, false, None, Direction::Right).is_err());
    assert!(graph.connect(999, node1, false, None, Direction::Right).is_err());
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
    assert_eq!(edge.traversal_flags, TraversalFlags::GHOST);
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

    assert_eq!(graph.node_count(), 2);
    let edge = graph.find_edge(node1, node2);
    assert!(edge.is_some());
    assert_eq!(edge.unwrap().direction, Direction::Right);
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
    assert!(result.is_err());
}

#[test]
fn should_error_on_duplicate_edge_without_replace() {
    let mut graph = create_test_graph();
    let result = graph.add_edge(0, 1, false, None, Direction::Right, TraversalFlags::ALL);
    assert!(result.is_err());
}

#[test]
fn should_allow_replacing_an_edge() {
    let mut graph = create_test_graph();
    let result = graph.add_edge(0, 1, true, Some(42.0), Direction::Right, TraversalFlags::ALL);
    assert!(result.is_ok());

    let edge = graph.find_edge(0, 1).unwrap();
    assert_eq!(edge.distance, 42.0);
}

#[test]
fn should_find_edge_between_nodes() {
    let graph = create_test_graph();
    let edge = graph.find_edge(0, 1);
    assert!(edge.is_some());
    assert_eq!(edge.unwrap().target, 1);

    let non_existent_edge = graph.find_edge(0, 99);
    assert!(non_existent_edge.is_none());
}
