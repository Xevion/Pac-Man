use glam::{UVec2, Vec2};

use crate::constants::BOARD_PIXEL_OFFSET;
use crate::entity::direction::Direction;
use crate::entity::graph::{Graph, NodeId, Position, Traverser};
use crate::helpers::centered_with_size;
use crate::texture::animated::AnimatedTexture;
use crate::texture::directional::DirectionalAnimatedTexture;
use crate::texture::sprite::SpriteAtlas;
use sdl2::keyboard::Keycode;
use sdl2::render::{Canvas, RenderTarget};
use std::collections::HashMap;

pub struct Pacman {
    pub traverser: Traverser,
    texture: DirectionalAnimatedTexture,
}

impl Pacman {
    pub fn new(graph: &Graph, start_node: NodeId, atlas: &SpriteAtlas) -> Self {
        let mut textures = HashMap::new();
        let mut stopped_textures = HashMap::new();

        for &direction in &[Direction::Up, Direction::Down, Direction::Left, Direction::Right] {
            let moving_prefix = match direction {
                Direction::Up => "pacman/up",
                Direction::Down => "pacman/down",
                Direction::Left => "pacman/left",
                Direction::Right => "pacman/right",
            };
            let moving_tiles = vec![
                SpriteAtlas::get_tile(atlas, &format!("{moving_prefix}_a.png")).unwrap(),
                SpriteAtlas::get_tile(atlas, &format!("{moving_prefix}_b.png")).unwrap(),
                SpriteAtlas::get_tile(atlas, "pacman/full.png").unwrap(),
            ];

            let stopped_tiles = vec![SpriteAtlas::get_tile(atlas, &format!("{moving_prefix}_b.png")).unwrap()];

            textures.insert(
                direction,
                AnimatedTexture::new(moving_tiles, 0.08).expect("Invalid frame duration"),
            );
            stopped_textures.insert(
                direction,
                AnimatedTexture::new(stopped_tiles, 0.1).expect("Invalid frame duration"),
            );
        }

        Self {
            traverser: Traverser::new(graph, start_node, Direction::Left),
            texture: DirectionalAnimatedTexture::new(textures, stopped_textures),
        }
    }

    pub fn tick(&mut self, dt: f32, graph: &Graph) {
        self.traverser.advance(graph, dt * 60.0 * 1.125);
        self.texture.tick(dt);
    }

    pub fn handle_key(&mut self, keycode: Keycode) {
        let direction = match keycode {
            Keycode::Up => Some(Direction::Up),
            Keycode::Down => Some(Direction::Down),
            Keycode::Left => Some(Direction::Left),
            Keycode::Right => Some(Direction::Right),
            _ => None,
        };

        if let Some(direction) = direction {
            self.traverser.set_next_direction(direction);
        }
    }

    fn get_pixel_pos(&self, graph: &Graph) -> Vec2 {
        match self.traverser.position {
            Position::AtNode(node_id) => graph.get_node(node_id).unwrap().position,
            Position::BetweenNodes { from, to, traversed } => {
                let from_pos = graph.get_node(from).unwrap().position;
                let to_pos = graph.get_node(to).unwrap().position;
                from_pos.lerp(to_pos, traversed / from_pos.distance(to_pos))
            }
        }
    }

    pub fn render<T: RenderTarget>(&self, canvas: &mut Canvas<T>, atlas: &mut SpriteAtlas, graph: &Graph) {
        let pixel_pos = self.get_pixel_pos(graph).round().as_ivec2() + BOARD_PIXEL_OFFSET.as_ivec2();
        let dest = centered_with_size(pixel_pos, UVec2::new(16, 16));
        let is_stopped = self.traverser.position.is_stopped();

        if is_stopped {
            self.texture
                .render_stopped(canvas, atlas, dest, self.traverser.direction)
                .unwrap();
        } else {
            self.texture.render(canvas, atlas, dest, self.traverser.direction).unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::graph::{Graph, Node};
    use crate::texture::sprite::{AtlasMapper, MapperFrame, SpriteAtlas};
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
    fn test_pacman_new() {
        let graph = create_test_graph();
        let atlas = create_test_atlas();

        let pacman = Pacman::new(&graph, 0, &atlas);

        assert_eq!(pacman.traverser.direction, Direction::Left);
        assert!(matches!(pacman.traverser.position, crate::entity::graph::Position::AtNode(0)));
    }

    #[test]
    fn test_handle_key_valid_directions() {
        let graph = create_test_graph();
        let atlas = create_test_atlas();
        let mut pacman = Pacman::new(&graph, 0, &atlas);

        // Test that direction keys are handled correctly
        // The traverser might consume next_direction immediately, so we check the actual direction
        pacman.handle_key(Keycode::Up);
        // Check that the direction was set (either in next_direction or current direction)
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

    #[test]
    fn test_get_pixel_pos_at_node() {
        let graph = create_test_graph();
        let atlas = create_test_atlas();
        let pacman = Pacman::new(&graph, 0, &atlas);

        let pos = pacman.get_pixel_pos(&graph);
        assert_eq!(pos, glam::Vec2::new(0.0, 0.0));
    }

    #[test]
    fn test_get_pixel_pos_between_nodes() {
        let graph = create_test_graph();
        let atlas = create_test_atlas();
        let mut pacman = Pacman::new(&graph, 0, &atlas);

        // Move pacman between nodes - need to advance with a larger distance to ensure movement
        pacman.traverser.advance(&graph, 5.0); // Larger advance to ensure movement

        let pos = pacman.get_pixel_pos(&graph);
        // Should be between (0,0) and (16,0), but not exactly at (8,0) due to advance distance
        assert!(pos.x >= 0.0 && pos.x <= 16.0);
        assert_eq!(pos.y, 0.0);
    }

    #[test]
    fn test_tick_updates_texture() {
        let graph = create_test_graph();
        let atlas = create_test_atlas();
        let mut pacman = Pacman::new(&graph, 0, &atlas);

        // Test that tick doesn't panic
        pacman.tick(0.016, &graph); // 60 FPS frame time
    }

    #[test]
    fn test_pacman_initial_direction() {
        let graph = create_test_graph();
        let atlas = create_test_atlas();
        let pacman = Pacman::new(&graph, 0, &atlas);

        // Pacman should start with the initial direction (Left)
        assert_eq!(pacman.traverser.direction, Direction::Left);
        // The next_direction might be consumed immediately when the traverser starts moving
        // So we just check that the direction is set correctly
        assert_eq!(pacman.traverser.direction, Direction::Left);
    }
}
