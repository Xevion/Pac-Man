use glam::{UVec2, Vec2};

use crate::constants::BOARD_PIXEL_OFFSET;
use crate::entity::direction::Direction;
use crate::entity::graph::{Edge, EdgePermissions, Graph, NodeId, Position, Traverser};
use crate::helpers::centered_with_size;
use crate::texture::animated::AnimatedTexture;
use crate::texture::directional::DirectionalAnimatedTexture;
use crate::texture::sprite::SpriteAtlas;
use sdl2::keyboard::Keycode;
use sdl2::render::{Canvas, RenderTarget};
use std::collections::HashMap;

fn can_pacman_traverse(edge: Edge) -> bool {
    matches!(edge.permissions, EdgePermissions::All)
}

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
            traverser: Traverser::new(graph, start_node, Direction::Left, &can_pacman_traverse),
            texture: DirectionalAnimatedTexture::new(textures, stopped_textures),
        }
    }

    pub fn tick(&mut self, dt: f32, graph: &Graph) {
        self.traverser.advance(graph, dt * 60.0 * 1.125, &can_pacman_traverse);
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
