use pacman::entity::direction::Direction;
use pacman::entity::graph::{EdgePermissions, Graph, Node};
use pacman::entity::traversal::{Position, Traverser};

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
        .add_edge(node1, node2, false, None, Direction::Right, EdgePermissions::GhostsOnly)
        .unwrap();

    let edge = graph.find_edge_in_direction(node1, Direction::Right).unwrap();
    assert_eq!(edge.permissions, EdgePermissions::GhostsOnly);
}

#[test]
fn test_traverser_basic() {
    let graph = create_test_graph();
    let mut traverser = Traverser::new(&graph, 0, Direction::Left, &|_| true);

    traverser.set_next_direction(Direction::Up);
    assert!(traverser.next_direction.is_some());
    assert_eq!(traverser.next_direction.unwrap().0, Direction::Up);
}

#[test]
fn test_traverser_advance() {
    let graph = create_test_graph();
    let mut traverser = Traverser::new(&graph, 0, Direction::Right, &|_| true);

    traverser.advance(&graph, 5.0, &|_| true);

    match traverser.position {
        Position::BetweenNodes { from, to, traversed } => {
            assert_eq!(from, 0);
            assert_eq!(to, 1);
            assert_eq!(traversed, 5.0);
        }
        _ => panic!("Expected to be between nodes"),
    }

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

#[test]
fn test_traverser_with_permissions() {
    let mut graph = Graph::new();
    let node1 = graph.add_node(Node {
        position: glam::Vec2::new(0.0, 0.0),
    });
    let node2 = graph.add_node(Node {
        position: glam::Vec2::new(16.0, 0.0),
    });

    graph
        .add_edge(node1, node2, false, None, Direction::Right, EdgePermissions::GhostsOnly)
        .unwrap();

    // Pacman can't traverse ghost-only edges
    let mut traverser = Traverser::new(&graph, node1, Direction::Right, &|edge| {
        matches!(edge.permissions, EdgePermissions::All)
    });

    traverser.advance(&graph, 5.0, &|edge| matches!(edge.permissions, EdgePermissions::All));

    // Should still be at the node since it can't traverse
    assert!(traverser.position.is_at_node());
}
