//! This module provides a simple animation system for textures.
use sdl2::{
    rect::Rect,
    render::{Canvas, Texture},
    video::Window,
};

use crate::direction::Direction;

/// An animated texture, which is a texture that is rendered as a series of
/// frames.
pub struct AnimatedTexture<'a> {
    raw_texture: Texture<'a>,
    /// The current tick of the animation.
    ticker: u32,
    /// Whether the animation is currently playing in reverse.
    reversed: bool,
    /// The offset of the texture, in pixels.
    offset: (i32, i32),
    /// The number of ticks per frame.
    ticks_per_frame: u32,
    /// The number of frames in the animation.
    frame_count: u32,
    /// The width of each frame, in pixels.
    frame_width: u32,
    /// The height of each frame, in pixels.
    frame_height: u32,
}

impl<'a> AnimatedTexture<'a> {
    /// Creates a new `AnimatedTexture`.
    ///
    /// # Arguments
    ///
    /// * `texture` - The texture to animate.
    /// * `ticks_per_frame` - The number of ticks to display each frame for.
    /// * `frame_count` - The number of frames in the animation.
    /// * `frame_width` - The width of each frame.
    /// * `frame_height` - The height of each frame.
    /// * `offset` - The offset of the texture, in pixels.
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

    /// Returns the current frame number.
    fn current_frame(&self) -> u32 {
        self.ticker / self.ticks_per_frame
    }

    /// Advances the animation by one tick.
    ///
    /// The animation will play forwards, then backwards, then forwards, and so on.
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

    /// Calculates the source rectangle for the given frame.
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

    /// Renders the animation to the canvas.
    ///
    /// # Arguments
    ///
    /// * `canvas` - The canvas to render to.
    /// * `position` - The position to render the animation at.
    /// * `direction` - The direction the animation is facing.
    pub fn render(
        &mut self,
        canvas: &mut Canvas<Window>,
        position: (i32, i32),
        direction: Direction,
    ) {
        self.render_static(canvas, position, direction, Some(self.current_frame()));
        self.tick();
    }

    /// Renders the animation to the canvas, but only ticks the animation until
    /// the given frame is reached.
    ///
    /// # Arguments
    ///
    /// * `canvas` - The canvas to render to.
    /// * `position` - The position to render the animation at.
    /// * `direction` - The direction the animation is facing.
    /// * `frame` - The frame to render until.
    pub fn render_until(
        &mut self,
        canvas: &mut Canvas<Window>,
        position: (i32, i32),
        direction: Direction,
        frame: u32,
    ) {
        // TODO: If the frame we're targeting is in the opposite direction (due
        // to self.reverse), we should pre-emptively reverse. This would require
        // a more complex ticking mechanism.
        let current = self.current_frame();
        self.render_static(canvas, position, direction, Some(current));

        if frame != current {
            self.tick();
        }
    }

    /// Renders a specific frame of the animation.
    ///
    /// # Arguments
    ///
    /// * `canvas` - The canvas to render to.
    /// * `position` - The position to render the animation at.
    /// * `direction` - The direction the animation is facing.
    /// * `frame` - The frame to render. If `None`, the current frame is used.
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
