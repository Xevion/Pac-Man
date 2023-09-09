use sdl2::{
    rect::Rect,
    render::{Canvas, Texture},
    video::Window,
};

use crate::direction::Direction;

pub struct AnimatedTexture<'a> {
    raw_texture: Texture<'a>,
    ticker: u32,
    reversed: bool,
    ticks_per_frame: u32,
    frame_count: u32,
    frame_width: u32,
    frame_height: u32,
}

impl<'a> AnimatedTexture<'a> {
    pub fn new(
        texture:Texture<'a>,
        ticks_per_frame: u32,
        frame_count: u32,
        frame_width: u32,
        frame_height: u32,
    ) -> Self {
        AnimatedTexture {
            raw_texture: texture,
            ticker: 0,
            reversed: false,
            ticks_per_frame,
            frame_count,
            frame_width,
            frame_height,
        }
    }

    fn current_frame(&self) -> u32 {
        self.ticker / self.ticks_per_frame
    }

    fn next_frame(&mut self) {
        if self.reversed {
            self.ticker -= 1;

            if self.ticker == 0 {
                self.reversed = !self.reversed;
            }
        } else {
            self.ticker += 1;

            if self.ticker > self.ticks_per_frame * self.frame_count {
                self.reversed = !self.reversed;
            }
        }
    }

    fn get_frame_rect(&self) -> Rect {
        Rect::new(
            self.current_frame() as i32 * self.frame_width as i32,
            0,
            self.frame_width,
            self.frame_height,
        )
    }

    pub fn render(&mut self, canvas: &mut Canvas<Window>, position: (i32, i32), direction: Direction) {
        let frame_rect = self.get_frame_rect();
        let position_rect = Rect::new(position.0, position.1, self.frame_width, self.frame_height);

        canvas
            .copy_ex(
                &self.raw_texture,
                Some(frame_rect),
                Some(position_rect),
                direction.angle(),
                None,
                false,
                false
            ).expect("Could not render texture on canvas");

            self.next_frame();
    }
}
