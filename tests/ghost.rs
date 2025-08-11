use pacman::entity::ghost::{Ghost, GhostType};
use pacman::entity::graph::Graph;
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
fn test_ghost_creation() {
    let graph = Graph::new();
    let atlas = create_test_atlas();

    let ghost = Ghost::new(&graph, 0, GhostType::Blinky, &atlas);

    assert_eq!(ghost.ghost_type, GhostType::Blinky);
    assert_eq!(ghost.traverser.position.from_node_id(), 0);
}
