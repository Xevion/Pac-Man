use pacman::entity::direction::Direction;
use pacman::entity::graph::{Graph, Node};
use pacman::entity::pacman::Pacman;
use pacman::texture::sprite::{AtlasMapper, MapperFrame, SpriteAtlas};
use sdl2::keyboard::Keycode;
use std::collections::HashMap;

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

fn create_test_atlas() -> SpriteAtlas {
    // Create a minimal test atlas with required tiles
    let mut frames = HashMap::new();
    frames.insert(
        "pacman/up_a.png".to_string(),
        MapperFrame {
            x: 0,
            y: 0,
            width: 16,
            height: 16,
        },
    );
    frames.insert(
        "pacman/up_b.png".to_string(),
        MapperFrame {
            x: 16,
            y: 0,
            width: 16,
            height: 16,
        },
    );
    frames.insert(
        "pacman/down_a.png".to_string(),
        MapperFrame {
            x: 32,
            y: 0,
            width: 16,
            height: 16,
        },
    );
    frames.insert(
        "pacman/down_b.png".to_string(),
        MapperFrame {
            x: 48,
            y: 0,
            width: 16,
            height: 16,
        },
    );
    frames.insert(
        "pacman/left_a.png".to_string(),
        MapperFrame {
            x: 64,
            y: 0,
            width: 16,
            height: 16,
        },
    );
    frames.insert(
        "pacman/left_b.png".to_string(),
        MapperFrame {
            x: 80,
            y: 0,
            width: 16,
            height: 16,
        },
    );
    frames.insert(
        "pacman/right_a.png".to_string(),
        MapperFrame {
            x: 96,
            y: 0,
            width: 16,
            height: 16,
        },
    );
    frames.insert(
        "pacman/right_b.png".to_string(),
        MapperFrame {
            x: 112,
            y: 0,
            width: 16,
            height: 16,
        },
    );
    frames.insert(
        "pacman/full.png".to_string(),
        MapperFrame {
            x: 128,
            y: 0,
            width: 16,
            height: 16,
        },
    );

    let mapper = AtlasMapper { frames };
    // Create a dummy texture (we won't actually render, just test the logic)
    let dummy_texture = unsafe { std::mem::zeroed() };
    SpriteAtlas::new(dummy_texture, mapper)
}

#[test]
fn test_handle_key_valid_directions() {
    let graph = create_test_graph();
    let atlas = create_test_atlas();
    let mut pacman = Pacman::new(&graph, 0, &atlas);

    // Test that direction keys are handled correctly
    pacman.handle_key(Keycode::Up);
    assert!(pacman.traverser.next_direction.is_some() || pacman.traverser.direction == Direction::Up);

    pacman.handle_key(Keycode::Down);
    assert!(pacman.traverser.next_direction.is_some() || pacman.traverser.direction == Direction::Down);

    pacman.handle_key(Keycode::Left);
    assert!(pacman.traverser.next_direction.is_some() || pacman.traverser.direction == Direction::Left);

    pacman.handle_key(Keycode::Right);
    assert!(pacman.traverser.next_direction.is_some() || pacman.traverser.direction == Direction::Right);
}

#[test]
fn test_handle_key_invalid_direction() {
    let graph = create_test_graph();
    let atlas = create_test_atlas();
    let mut pacman = Pacman::new(&graph, 0, &atlas);

    let original_direction = pacman.traverser.direction;
    let original_next_direction = pacman.traverser.next_direction;

    // Test invalid key
    pacman.handle_key(Keycode::Space);

    // Should not change direction
    assert_eq!(pacman.traverser.direction, original_direction);
    assert_eq!(pacman.traverser.next_direction, original_next_direction);
}
