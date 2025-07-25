//! A texture that changes based on the direction of an entity.
use crate::entity::direction::Direction;
use crate::texture::sprite::AtlasTile;
use anyhow::Result;
use glam::IVec2;
use sdl2::render::WindowCanvas;

pub struct DirectionalAnimatedTexture {
    pub up: Vec<AtlasTile>,
    pub down: Vec<AtlasTile>,
    pub left: Vec<AtlasTile>,
    pub right: Vec<AtlasTile>,
    pub ticker: u32,
    pub ticks_per_frame: u32,
}

impl DirectionalAnimatedTexture {
    pub fn new(
        up: Vec<AtlasTile>,
        down: Vec<AtlasTile>,
        left: Vec<AtlasTile>,
        right: Vec<AtlasTile>,
        ticks_per_frame: u32,
    ) -> Self {
        Self {
            up,
            down,
            left,
            right,
            ticker: 0,
            ticks_per_frame,
        }
    }

    pub fn tick(&mut self) {
        self.ticker += 1;
    }

    pub fn render(&mut self, canvas: &mut WindowCanvas, dest: sdl2::rect::Rect, direction: Direction) -> Result<()> {
        let frames = match direction {
            Direction::Up => &self.up,
            Direction::Down => &self.down,
            Direction::Left => &self.left,
            Direction::Right => &self.right,
        };

        let frame_index = (self.ticker / self.ticks_per_frame) as usize % frames.len();
        let tile = &frames[frame_index];

        tile.render(canvas, dest)
    }
}
