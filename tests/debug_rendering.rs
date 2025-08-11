use glam::Vec2;
use pacman::entity::graph::{Graph, Node};
use pacman::map::render::MapRenderer;

#[test]
fn test_find_nearest_node() {
    let mut graph = Graph::new();

    // Add some test nodes
    let node1 = graph.add_node(Node {
        position: Vec2::new(10.0, 10.0),
    });
    let node2 = graph.add_node(Node {
        position: Vec2::new(50.0, 50.0),
    });
    let node3 = graph.add_node(Node {
        position: Vec2::new(100.0, 100.0),
    });

    // Test cursor near node1
    let cursor_pos = Vec2::new(12.0, 8.0);
    let nearest = MapRenderer::find_nearest_node(&graph, cursor_pos);
    assert_eq!(nearest, Some(node1));

    // Test cursor near node2
    let cursor_pos = Vec2::new(45.0, 55.0);
    let nearest = MapRenderer::find_nearest_node(&graph, cursor_pos);
    assert_eq!(nearest, Some(node2));

    // Test cursor near node3
    let cursor_pos = Vec2::new(98.0, 102.0);
    let nearest = MapRenderer::find_nearest_node(&graph, cursor_pos);
    assert_eq!(nearest, Some(node3));
}
