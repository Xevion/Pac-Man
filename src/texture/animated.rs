use smallvec::SmallVec;

use crate::error::{AnimatedTextureError, GameError, GameResult, TextureError};
use crate::texture::sprite::AtlasTile;

/// Frame-based animation system for cycling through multiple sprite tiles.
///
/// Manages automatic frame progression based on elapsed ticks.
/// Uses a tick banking system to ensure consistent animation speed regardless of frame rate variations.
#[derive(Debug, Clone)]
pub struct AnimatedTexture {
    /// Sequence of sprite tiles that make up the animation frames
    tiles: SmallVec<[AtlasTile; 4]>,
    /// Duration each frame should be displayed (in ticks)
    frame_duration: u16,
    /// Index of the currently active frame in the tiles vector
    current_frame: usize,
    /// Accumulated ticks since the last frame change (for smooth timing)
    time_bank: u16,
}

impl AnimatedTexture {
    pub fn new(tiles: SmallVec<[AtlasTile; 4]>, frame_duration: u16) -> GameResult<Self> {
        if frame_duration == 0 {
            return Err(GameError::Texture(TextureError::Animated(
                AnimatedTextureError::InvalidFrameDuration(frame_duration),
            )));
        }

        Ok(Self {
            tiles,
            frame_duration,
            current_frame: 0,
            time_bank: 0,
        })
    }

    /// Advances the animation by the specified number of ticks with automatic frame cycling.
    ///
    /// Accumulates ticks in the time bank and progresses through frames when enough
    /// ticks have elapsed. Supports frame rates independent of game frame rate by
    /// potentially advancing multiple frames in a single call if `ticks` is large.
    /// Animation loops automatically when reaching the final frame.
    ///
    /// # Arguments
    ///
    /// * `ticks` - Number of ticks elapsed since the last update
    pub fn tick(&mut self, ticks: u16) {
        self.time_bank += ticks;
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
    pub fn time_bank(&self) -> u16 {
        self.time_bank
    }

    /// Returns the frame duration.
    #[allow(dead_code)]
    pub fn frame_duration(&self) -> u16 {
        self.frame_duration
    }

    /// Returns the number of tiles in the animation.
    #[allow(dead_code)]
    pub fn tiles_len(&self) -> usize {
        self.tiles.len()
    }

    /// Resets the animation to the first frame and clears the time bank.
    /// Useful for synchronizing animations when they are assigned.
    pub fn reset(&mut self) {
        self.current_frame = 0;
        self.time_bank = 0;
    }
}
