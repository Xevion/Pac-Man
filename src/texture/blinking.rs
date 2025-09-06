#![allow(dead_code)]
use crate::texture::sprite::AtlasTile;

#[derive(Clone)]
pub struct BlinkingTexture {
    tile: AtlasTile,
    interval_ticks: u32,
    tick_timer: u32,
    is_on: bool,
}

impl BlinkingTexture {
    pub fn new(tile: AtlasTile, interval_ticks: u32) -> Self {
        Self {
            tile,
            interval_ticks,
            tick_timer: 0,
            is_on: true,
        }
    }

    pub fn tick(&mut self, delta_ticks: u32) {
        self.tick_timer += delta_ticks;

        // Handle zero interval case (immediate toggling)
        if self.interval_ticks == 0 {
            // With zero interval, any positive ticks should toggle
            if delta_ticks > 0 {
                self.is_on = !self.is_on;
            }
            return;
        }

        // Calculate how many complete intervals have passed
        let complete_intervals = self.tick_timer / self.interval_ticks;

        // Update the timer to the remainder after complete intervals
        self.tick_timer %= self.interval_ticks;

        // Toggle for each complete interval, but since toggling twice is a no-op,
        // we only need to toggle if the count is odd
        if complete_intervals % 2 == 1 {
            self.is_on = !self.is_on;
        }
    }

    pub fn is_on(&self) -> bool {
        self.is_on
    }

    pub fn tile(&self) -> &AtlasTile {
        &self.tile
    }

    // Helper methods for testing
    pub fn tick_timer(&self) -> u32 {
        self.tick_timer
    }

    pub fn interval_ticks(&self) -> u32 {
        self.interval_ticks
    }
}
