use pacman::entity::direction::Direction;
use pacman::entity::ghost::{Ghost, GhostType};
use pacman::entity::graph::{Graph, Node};
use pacman::texture::sprite::{AtlasMapper, MapperFrame, SpriteAtlas};
use std::collections::HashMap;

fn create_test_atlas() -> SpriteAtlas {
    let mut frames = HashMap::new();
    let directions = ["up", "down", "left", "right"];
    let ghost_types = ["blinky", "pinky", "inky", "clyde"];

    for ghost_type in &ghost_types {
        for (i, dir) in directions.iter().enumerate() {
            frames.insert(
                format!("ghost/{}/{}_{}.png", ghost_type, dir, "a"),
                MapperFrame {
                    x: i as u16 * 16,
                    y: 0,
                    width: 16,
                    height: 16,
                },
            );
            frames.insert(
                format!("ghost/{}/{}_{}.png", ghost_type, dir, "b"),
                MapperFrame {
                    x: i as u16 * 16,
                    y: 16,
                    width: 16,
                    height: 16,
                },
            );
        }
    }

    let mapper = AtlasMapper { frames };
    let dummy_texture = unsafe { std::mem::zeroed() };
    SpriteAtlas::new(dummy_texture, mapper)
}

#[test]
fn test_ghost_pathfinding() {
    // Create a simple test graph
    let mut graph = Graph::new();

    // Add nodes in a simple line: 0 -> 1 -> 2
    let node0 = graph.add_node(Node {
        position: glam::Vec2::new(0.0, 0.0),
    });
    let node1 = graph.add_node(Node {
        position: glam::Vec2::new(10.0, 0.0),
    });
    let node2 = graph.add_node(Node {
        position: glam::Vec2::new(20.0, 0.0),
    });

    // Connect the nodes
    graph.connect(node0, node1, false, None, Direction::Right).unwrap();
    graph.connect(node1, node2, false, None, Direction::Right).unwrap();

    // Create a test atlas for the ghost
    let atlas = create_test_atlas();

    // Create a ghost at node 0
    let ghost = Ghost::new(&graph, node0, GhostType::Blinky, &atlas).unwrap();

    // Test pathfinding from node 0 to node 2
    let path = ghost.calculate_path_to_target(&graph, node2);

    assert!(path.is_ok());
    let path = path.unwrap();
    assert!(
        path == vec![node0, node1, node2] || path == vec![node2, node1, node0],
        "Path was not what was expected"
    );
}

#[test]
fn test_ghost_pathfinding_no_path() {
    // Create a test graph with disconnected components
    let mut graph = Graph::new();

    let node0 = graph.add_node(Node {
        position: glam::Vec2::new(0.0, 0.0),
    });
    let node1 = graph.add_node(Node {
        position: glam::Vec2::new(10.0, 0.0),
    });

    // Don't connect the nodes
    let atlas = create_test_atlas();
    let ghost = Ghost::new(&graph, node0, GhostType::Blinky, &atlas).unwrap();

    // Test pathfinding when no path exists
    let path = ghost.calculate_path_to_target(&graph, node1);

    assert!(path.is_err());
}

#[test]
fn test_ghost_debug_colors() {
    let atlas = create_test_atlas();
    let mut graph = Graph::new();
    let node = graph.add_node(Node {
        position: glam::Vec2::new(0.0, 0.0),
    });

    let blinky = Ghost::new(&graph, node, GhostType::Blinky, &atlas).unwrap();
    let pinky = Ghost::new(&graph, node, GhostType::Pinky, &atlas).unwrap();
    let inky = Ghost::new(&graph, node, GhostType::Inky, &atlas).unwrap();
    let clyde = Ghost::new(&graph, node, GhostType::Clyde, &atlas).unwrap();

    // Test that each ghost has a different debug color
    let colors = std::collections::HashSet::from([
        blinky.debug_color(),
        pinky.debug_color(),
        inky.debug_color(),
        clyde.debug_color(),
    ]);
    assert_eq!(colors.len(), 4, "All ghost colors should be unique");
}
