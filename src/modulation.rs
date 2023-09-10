pub struct SpeedModulator {
    tick_count: u32,
    ticks_left: u32,
}

impl SpeedModulator {
    pub fn new(percent: f32) -> Self {
        let ticks_required: u32 = (1f32 / (1f32 - percent)).round() as u32;

        SpeedModulator {
            tick_count: ticks_required,
            ticks_left: ticks_required,
        }
    }

    pub fn next(&mut self) -> bool {
        self.ticks_left -= 1;

        if self.ticks_left == 0 {
            self.ticks_left = self.tick_count;
            false
        } else {
            true
        }
    }
}
