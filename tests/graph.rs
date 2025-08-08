use pacman::entity::direction::Direction;
use pacman::entity::graph::{Graph, Node, Position, Traverser};

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
fn test_graph_connect() {
    let mut graph = Graph::new();
    let node1 = graph.add_node(Node {
        position: glam::Vec2::new(0.0, 0.0),
    });
    let node2 = graph.add_node(Node {
        position: glam::Vec2::new(16.0, 0.0),
    });

    let result = graph.connect(node1, node2, false, None, Direction::Right);
    assert!(result.is_ok());

    // Check that edges were added in both directions
    let edge1 = graph.find_edge_in_direction(node1, Direction::Right);
    let edge2 = graph.find_edge_in_direction(node2, Direction::Left);

    assert!(edge1.is_some());
    assert!(edge2.is_some());
    assert_eq!(edge1.unwrap().target, node2);
    assert_eq!(edge2.unwrap().target, node1);
}

#[test]
fn test_graph_connect_invalid_nodes() {
    let mut graph = Graph::new();
    let node1 = graph.add_node(Node {
        position: glam::Vec2::new(0.0, 0.0),
    });

    // Try to connect to non-existent node
    let result = graph.connect(node1, 999, false, None, Direction::Right);
    assert!(result.is_err());

    // Try to connect from non-existent node
    let result = graph.connect(999, node1, false, None, Direction::Right);
    assert!(result.is_err());
}

#[test]
fn test_graph_find_edge() {
    let mut graph = Graph::new();
    let node1 = graph.add_node(Node {
        position: glam::Vec2::new(0.0, 0.0),
    });
    let node2 = graph.add_node(Node {
        position: glam::Vec2::new(16.0, 0.0),
    });

    graph.connect(node1, node2, false, None, Direction::Right).unwrap();

    let edge = graph.find_edge(node1, node2);
    assert!(edge.is_some());
    assert_eq!(edge.unwrap().target, node2);

    // Test non-existent edge
    assert!(graph.find_edge(node1, 999).is_none());
}

#[test]
fn test_graph_find_edge_in_direction() {
    let mut graph = Graph::new();
    let node1 = graph.add_node(Node {
        position: glam::Vec2::new(0.0, 0.0),
    });
    let node2 = graph.add_node(Node {
        position: glam::Vec2::new(16.0, 0.0),
    });

    graph.connect(node1, node2, false, None, Direction::Right).unwrap();

    let edge = graph.find_edge_in_direction(node1, Direction::Right);
    assert!(edge.is_some());
    assert_eq!(edge.unwrap().target, node2);

    // Test non-existent direction
    assert!(graph.find_edge_in_direction(node1, Direction::Up).is_none());
}

#[test]
fn test_traverser_set_next_direction() {
    let graph = create_test_graph();
    let mut traverser = Traverser::new(&graph, 0, Direction::Left, &|_| true);

    traverser.set_next_direction(Direction::Up);
    assert!(traverser.next_direction.is_some());
    assert_eq!(traverser.next_direction.unwrap().0, Direction::Up);

    // Setting same direction should not change anything
    traverser.set_next_direction(Direction::Up);
    assert_eq!(traverser.next_direction.unwrap().0, Direction::Up);
}

#[test]
fn test_traverser_advance_at_node() {
    let graph = create_test_graph();
    let mut traverser = Traverser::new(&graph, 0, Direction::Right, &|_| true);

    // Should start moving in the initial direction
    traverser.advance(&graph, 5.0, &|_| true);

    match traverser.position {
        Position::BetweenNodes { from, to, traversed } => {
            assert_eq!(from, 0);
            assert_eq!(to, 1);
            assert_eq!(traversed, 5.0);
        }
        _ => panic!("Expected to be between nodes"),
    }
}

#[test]
fn test_traverser_advance_between_nodes() {
    let graph = create_test_graph();
    let mut traverser = Traverser::new(&graph, 0, Direction::Right, &|_| true);

    // Move to between nodes
    traverser.advance(&graph, 5.0, &|_| true);

    // Advance further
    traverser.advance(&graph, 3.0, &|_| true);

    match traverser.position {
        Position::BetweenNodes { from, to, traversed } => {
            assert_eq!(from, 0);
            assert_eq!(to, 1);
            assert_eq!(traversed, 8.0);
        }
        _ => panic!("Expected to be between nodes"),
    }
}
