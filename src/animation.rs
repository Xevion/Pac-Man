//! This module provides a simple animation system for textures.
use sdl2::{
    rect::Rect,
    render::{Canvas, Texture},
    video::Window,
};

use crate::direction::Direction;

/// An animated texture, which is a texture that is rendered as a series of
/// frames.
///
/// This struct manages the state of an animated texture, including the current
/// frame and the number of frames in the animation.
pub struct AnimatedTexture<'a> {
    // Parameters
    raw_texture: Texture<'a>,
    offset: (i32, i32),
    ticks_per_frame: u32,
    frame_count: u32,
    width: u32,
    height: u32,
    // State
    ticker: u32,
    reversed: bool,
}

impl<'a> AnimatedTexture<'a> {
    pub fn new(
        texture: Texture<'a>,
        ticks_per_frame: u32,
        frame_count: u32,
        width: u32,
        height: u32,
        offset: Option<(i32, i32)>,
    ) -> Self {
        AnimatedTexture {
            raw_texture: texture,
            ticker: 0,
            reversed: false,
            ticks_per_frame,
            frame_count,
            width,
            height,
            offset: offset.unwrap_or((0, 0)),
        }
    }

    fn current_frame(&self) -> u32 {
        self.ticker / self.ticks_per_frame
    }

    /// Advances the animation by one tick.
    ///
    /// This method updates the internal ticker that tracks the current frame
    /// of the animation. The animation automatically reverses direction when
    /// it reaches the end, creating a ping-pong effect.
    ///
    /// When `reversed` is `false`, the ticker increments until it reaches
    /// the total number of ticks for all frames, then reverses direction.
    /// When `reversed` is `true`, the ticker decrements until it reaches 0,
    /// then reverses direction again.
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

    /// Gets the source rectangle for a specific frame of the animated texture.
    ///
    /// This method calculates the position and dimensions of a frame within the
    /// texture atlas. Frames are arranged horizontally in a single row, so the
    /// rectangle's x-coordinate is calculated by multiplying the frame index
    /// by the frame width.
    ///
    /// # Arguments
    ///
    /// * `frame` - The frame index to get the rectangle for (0-based)
    ///
    /// # Returns
    ///
    /// A `Rect` representing the source rectangle for the specified frame
    fn get_frame_rect(&self, frame: u32) -> Option<Rect> {
        if frame >= self.frame_count {
            return None;
        }

        Some(Rect::new(
            frame as i32 * self.width as i32,
            0,
            self.width,
            self.height,
        ))
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

    /// Renders a specific frame of the animated texture to the canvas.
    ///
    /// This method renders a static frame without advancing the animation ticker.
    /// It's useful for displaying a specific frame, such as when an entity is stopped
    /// or when you want to manually control which frame is displayed.
    ///
    /// # Arguments
    ///
    /// * `canvas` - The SDL canvas to render to
    /// * `position` - The pixel position where the texture should be rendered
    /// * `direction` - The direction to rotate the texture based on entity facing
    /// * `frame` - Optional specific frame to render. If `None`, uses the current frame
    ///
    /// # Panics
    ///
    /// Panics if the specified frame is out of bounds for this texture.
    pub fn render_static(
        &mut self,
        canvas: &mut Canvas<Window>,
        position: (i32, i32),
        direction: Direction,
        frame: Option<u32>,
    ) {
        let texture_source_frame_rect =
            self.get_frame_rect(frame.unwrap_or_else(|| self.current_frame()));
        let canvas_destination_rect = Rect::new(
            position.0 + self.offset.0,
            position.1 + self.offset.1,
            self.width,
            self.height,
        );

        canvas
            .copy_ex(
                &self.raw_texture,
                texture_source_frame_rect,
                Some(canvas_destination_rect),
                direction.angle(),
                None,
                false,
                false,
            )
            .expect("Could not render texture on canvas");
    }

    /// Sets the color modulation for the texture.
    pub fn set_color_modulation(&mut self, r: u8, g: u8, b: u8) {
        self.raw_texture.set_color_mod(r, g, b);
    }
}
