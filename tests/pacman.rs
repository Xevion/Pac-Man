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
    let mut frames = HashMap::new();
    let directions = ["up", "down", "left", "right"];

    for (i, dir) in directions.iter().enumerate() {
        frames.insert(
            format!("pacman/{dir}_a.png"),
            MapperFrame {
                x: i as u16 * 16,
                y: 0,
                width: 16,
                height: 16,
            },
        );
        frames.insert(
            format!("pacman/{dir}_b.png"),
            MapperFrame {
                x: i as u16 * 16,
                y: 16,
                width: 16,
                height: 16,
            },
        );
    }

    frames.insert(
        "pacman/full.png".to_string(),
        MapperFrame {
            x: 64,
            y: 0,
            width: 16,
            height: 16,
        },
    );

    let mapper = AtlasMapper { frames };
    let dummy_texture = unsafe { std::mem::zeroed() };
    SpriteAtlas::new(dummy_texture, mapper)
}

#[test]
fn test_pacman_creation() {
    let graph = create_test_graph();
    let atlas = create_test_atlas();
    let pacman = Pacman::new(&graph, 0, &atlas).unwrap();

    assert!(pacman.traverser.position.is_at_node());
    assert_eq!(pacman.traverser.direction, Direction::Left);
}

#[test]
fn test_pacman_key_handling() {
    let graph = create_test_graph();
    let atlas = create_test_atlas();
    let mut pacman = Pacman::new(&graph, 0, &atlas).unwrap();

    let test_cases = [
        (Keycode::Up, Direction::Up),
        (Keycode::Down, Direction::Down),
        (Keycode::Left, Direction::Left),
        (Keycode::Right, Direction::Right),
    ];

    for (key, expected_direction) in test_cases {
        pacman.handle_key(key);
        assert!(pacman.traverser.next_direction.is_some() || pacman.traverser.direction == expected_direction);
    }
}

#[test]
fn test_pacman_invalid_key() {
    let graph = create_test_graph();
    let atlas = create_test_atlas();
    let mut pacman = Pacman::new(&graph, 0, &atlas).unwrap();

    let original_direction = pacman.traverser.direction;
    let original_next_direction = pacman.traverser.next_direction;

    pacman.handle_key(Keycode::Space);
    assert_eq!(pacman.traverser.direction, original_direction);
    assert_eq!(pacman.traverser.next_direction, original_next_direction);
}
