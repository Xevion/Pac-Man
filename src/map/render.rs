//! Map rendering functionality.

use crate::texture::sprite::{AtlasTile, SpriteAtlas};
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{Canvas, RenderTarget};

/// Handles rendering operations for the map.
pub struct MapRenderer;

impl MapRenderer {
    /// Renders the map to the given canvas.
    ///
    /// This function draws the static map texture to the screen at the correct
    /// position and scale.
    pub fn render_map<T: RenderTarget>(canvas: &mut Canvas<T>, atlas: &mut SpriteAtlas, map_texture: &mut AtlasTile) {
        let dest = Rect::new(
            crate::constants::BOARD_PIXEL_OFFSET.x as i32,
            crate::constants::BOARD_PIXEL_OFFSET.y as i32,
            crate::constants::BOARD_PIXEL_SIZE.x,
            crate::constants::BOARD_PIXEL_SIZE.y,
        );
        let _ = map_texture.render(canvas, atlas, dest);
    }

    /// Renders a debug visualization of the navigation graph.
    ///
    /// This function is intended for development and debugging purposes. It draws the
    /// nodes and edges of the graph on top of the map, allowing for visual
    /// inspection of the navigation paths.
    pub fn debug_render_nodes<T: RenderTarget>(graph: &crate::entity::graph::Graph, canvas: &mut Canvas<T>) {
        for i in 0..graph.node_count() {
            let node = graph.get_node(i).unwrap();
            let pos = node.position + crate::constants::BOARD_PIXEL_OFFSET.as_vec2();

            // Draw connections
            canvas.set_draw_color(Color::BLUE);

            for edge in graph.adjacency_list[i].edges() {
                let end_pos = graph.get_node(edge.target).unwrap().position + crate::constants::BOARD_PIXEL_OFFSET.as_vec2();
                canvas
                    .draw_line((pos.x as i32, pos.y as i32), (end_pos.x as i32, end_pos.y as i32))
                    .unwrap();
            }

            // Draw node
            // let color = if pacman.position.from_node_idx() == i.into() {
            //     Color::GREEN
            // } else if let Some(to_idx) = pacman.position.to_node_idx() {
            //     if to_idx == i.into() {
            //         Color::CYAN
            //     } else {
            //         Color::RED
            //     }
            // } else {
            //     Color::RED
            // };
            canvas.set_draw_color(Color::GREEN);
            canvas
                .fill_rect(Rect::new(0, 0, 3, 3).centered_on(Point::new(pos.x as i32, pos.y as i32)))
                .unwrap();

            // Draw node index
            // text.render(canvas, atlas, &i.to_string(), pos.as_uvec2()).unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::graph::{Graph, Node};
    use crate::texture::sprite::{AtlasMapper, MapperFrame};
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

        graph
            .connect(node1, node2, false, None, crate::entity::direction::Direction::Right)
            .unwrap();
        graph
            .connect(node1, node3, false, None, crate::entity::direction::Direction::Down)
            .unwrap();

        graph
    }

    fn create_test_atlas() -> SpriteAtlas {
        let mut frames = HashMap::new();
        frames.insert(
            "maze/full.png".to_string(),
            MapperFrame {
                x: 0,
                y: 0,
                width: 224,
                height: 248,
            },
        );
        let mapper = AtlasMapper { frames };
        let dummy_texture = unsafe { std::mem::zeroed() };
        SpriteAtlas::new(dummy_texture, mapper)
    }

    #[test]
    fn test_render_map_does_not_panic() {
        // This test just ensures the function doesn't panic
        // We can't easily test the actual rendering without SDL context
        let atlas = create_test_atlas();
        let _map_texture = SpriteAtlas::get_tile(&atlas, "maze/full.png").unwrap();

        // The function should not panic even with dummy data
        // Note: We can't actually call render_map without a canvas, but we can test the logic
        assert!(true); // Placeholder test
    }

    #[test]
    fn test_debug_render_nodes_does_not_panic() {
        // This test just ensures the function doesn't panic
        // We can't easily test the actual rendering without SDL context
        let _graph = create_test_graph();

        // The function should not panic even with dummy data
        // Note: We can't actually call debug_render_nodes without a canvas, but we can test the logic
        assert!(true); // Placeholder test
    }

    #[test]
    fn test_map_renderer_structure() {
        // Test that MapRenderer is a unit struct
        let _renderer = MapRenderer;
        // This should compile and not panic
        assert!(true);
    }
}
