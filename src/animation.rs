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
    offset: (i32, i32),
    ticks_per_frame: u32,
    frame_count: u32,
    frame_width: u32,
    frame_height: u32,
}

impl<'a> AnimatedTexture<'a> {
    pub fn new(
        texture: Texture<'a>,
        ticks_per_frame: u32,
        frame_count: u32,
        frame_width: u32,
        frame_height: u32,
        offset: Option<(i32, i32)>,
    ) -> Self {
        AnimatedTexture {
            raw_texture: texture,
            ticker: 0,
            reversed: false,
            ticks_per_frame,
            frame_count,
            frame_width,
            frame_height,
            offset: offset.unwrap_or((0, 0)),
        }
    }

    // Get the current frame number
    fn current_frame(&self) -> u32 {
        self.ticker / self.ticks_per_frame
    }

    // Move to the next frame. If we are at the end of the animation, reverse the direction
    pub fn tick(&mut self) {
        if self.reversed {
            self.ticker -= 1;

            if self.ticker == 0 {
                self.reversed = !self.reversed;
            }
        } else {
            self.ticker += 1;

            if self.ticker + 1 == self.ticks_per_frame * self.frame_count {
                self.reversed = !self.reversed;
            }
        }
    }

    // Calculate the frame rect (portion of the texture to render) for the given frame.
    fn get_frame_rect(&self, frame: u32) -> Rect {
        if frame >= self.frame_count {
            panic!("Frame {} is out of bounds for this texture", frame);
        }

        Rect::new(
            frame as i32 * self.frame_width as i32,
            0,
            self.frame_width,
            self.frame_height,
        )
    }

    pub fn render(
        &mut self,
        canvas: &mut Canvas<Window>,
        position: (i32, i32),
        direction: Direction,
    ) {
        self.render_static(canvas, position, direction, Some(self.current_frame()));
        self.tick();
    }
    
    pub fn render_static(
        &mut self,
        canvas: &mut Canvas<Window>,
        position: (i32, i32),
        direction: Direction,
        frame: Option<u32>,
    ) {
        let frame_rect = self.get_frame_rect(frame.unwrap_or(self.current_frame()));
        let position_rect = Rect::new(
            position.0 + self.offset.0,
            position.1 + self.offset.1,
            self.frame_width,
            self.frame_height,
        );

        canvas
            .copy_ex(
                &self.raw_texture,
                Some(frame_rect),
                Some(position_rect),
                direction.angle(),
                None,
                false,
                false,
            )
            .expect("Could not render texture on canvas");
    }
}
