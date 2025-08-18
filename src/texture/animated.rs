use crate::error::{AnimatedTextureError, GameError, GameResult, TextureError};
use crate::texture::sprite::AtlasTile;

/// Frame-based animation system for cycling through multiple sprite tiles.
///
/// Manages automatic frame progression based on elapsed time.
/// Uses a time banking system to ensure consistent animation speed regardless of frame rate variations.
#[derive(Debug, Clone)]
pub struct AnimatedTexture {
    /// Sequence of sprite tiles that make up the animation frames
    tiles: Vec<AtlasTile>,
    /// Duration each frame should be displayed (in seconds)
    frame_duration: f32,
    /// Index of the currently active frame in the tiles vector
    current_frame: usize,
    /// Accumulated time since the last frame change (for smooth timing)
    time_bank: f32,
}

impl AnimatedTexture {
    pub fn new(tiles: Vec<AtlasTile>, frame_duration: f32) -> GameResult<Self> {
        if frame_duration <= 0.0 {
            return Err(GameError::Texture(TextureError::Animated(
                AnimatedTextureError::InvalidFrameDuration(frame_duration),
            )));
        }

        Ok(Self {
            tiles,
            frame_duration,
            current_frame: 0,
            time_bank: 0.0,
        })
    }

    /// Advances the animation by the specified time delta with automatic frame cycling.
    ///
    /// Accumulates time in the time bank and progresses through frames when enough
    /// time has elapsed. Supports frame rates independent of game frame rate by
    /// potentially advancing multiple frames in a single call if `dt` is large.
    /// Animation loops automatically when reaching the final frame.
    ///
    /// # Arguments
    ///
    /// * `dt` - Time elapsed since the last tick (typically frame delta time)
    pub fn tick(&mut self, dt: f32) {
        self.time_bank += dt;
        while self.time_bank >= self.frame_duration {
            self.time_bank -= self.frame_duration;
            self.current_frame = (self.current_frame + 1) % self.tiles.len();
        }
    }

    pub fn current_tile(&self) -> &AtlasTile {
        &self.tiles[self.current_frame]
    }

    /// Returns the current frame index.
    #[allow(dead_code)]
    pub fn current_frame(&self) -> usize {
        self.current_frame
    }

    /// Returns the time bank.
    #[allow(dead_code)]
    pub fn time_bank(&self) -> f32 {
        self.time_bank
    }

    /// Returns the frame duration.
    #[allow(dead_code)]
    pub fn frame_duration(&self) -> f32 {
        self.frame_duration
    }

    /// Returns the number of tiles in the animation.
    #[allow(dead_code)]
    pub fn tiles_len(&self) -> usize {
        self.tiles.len()
    }
}
