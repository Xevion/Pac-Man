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
    fn new(percent: f32) -> Self;
    fn next(&mut self) -> bool;
}

pub struct SimpleTickModulator {
    tick_count: u32,
    ticks_left: u32,
}

// TODO: Add tests
// TODO: Look into average precision, binary code modulation strategy
impl TickModulator for SimpleTickModulator {
    fn new(percent: f32) -> Self {
        let ticks_required: u32 = (1f32 / (1f32 - percent)).round() as u32;

        SimpleTickModulator {
            tick_count: ticks_required,
            ticks_left: ticks_required,
        }
    }

    fn next(&mut self) -> bool {
        self.ticks_left -= 1;

        // Return whether or not we should skip this tick
        if self.ticks_left == 0 {
            // We've reached the tick to skip, reset the counter
            self.ticks_left = self.tick_count;
            false
        } else {
            true
        }
    }
}
