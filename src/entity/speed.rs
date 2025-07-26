//! This module provides a tick modulator, which can be used to slow down
//! operations by a percentage.
/// A tick modulator allows you to slow down operations by a percentage.
///
/// Unfortunately, switching to floating point numbers for entities can induce floating point errors, slow down calculations
/// and make the game less deterministic. This is why we use a speed modulator instead.
/// Additionally, with small integers, lowering the speed by a percentage is not possible. For example, if we have a speed of 2,
/// and we want to slow it down by 10%, we would need to slow it down by 0.2. However, since we are using integers, we can't.
/// The only amount you can slow it down by is 1, which is 50% of the speed.
///
/// The basic principle of the Speed Modulator is to instead 'skip' movement ticks every now and then.
/// At 60 ticks per second, skips could happen several times per second, or once every few seconds.
/// Whatever it be, as long as the tick rate is high enough, the human eye will not be able to tell the difference.
///
/// For example, if we want to slow down the speed by 10%, we would need to skip every 10th tick.
pub trait TickModulator {
    /// Creates a new tick modulator.
    ///
    /// # Arguments
    ///
    /// * `percent` - The percentage to slow down by, from 0.0 to 1.0.
    fn new(percent: f32) -> Self;
    /// Returns whether or not the operation should be performed on this tick.
    fn next(&mut self) -> bool;
    fn set_speed(&mut self, speed: f32);
}

/// A simple tick modulator that skips every Nth tick.
pub struct SimpleTickModulator {
    accumulator: f32,
    pixels_per_tick: f32,
}

// TODO: Add tests for the tick modulator to ensure that it is working correctly.
// TODO: Look into average precision and binary code modulation strategies to see
// if they would be a better fit for this use case.
impl SimpleTickModulator {
    pub fn new(pixels_per_tick: f32) -> Self {
        Self {
            accumulator: 0f32,
            pixels_per_tick: pixels_per_tick * 0.47,
        }
    }
    pub fn set_speed(&mut self, pixels_per_tick: f32) {
        self.pixels_per_tick = pixels_per_tick;
    }
    pub fn next(&mut self) -> bool {
        self.accumulator += self.pixels_per_tick;
        if self.accumulator >= 1f32 {
            self.accumulator -= 1f32;
            true
        } else {
            false
        }
    }
}
