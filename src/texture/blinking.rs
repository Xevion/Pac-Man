use crate::texture::sprite::AtlasTile;

#[derive(Clone)]
pub struct BlinkingTexture {
    tile: AtlasTile,
    blink_duration: f32,
    time_bank: f32,
    is_on: bool,
}

impl BlinkingTexture {
    pub fn new(tile: AtlasTile, blink_duration: f32) -> Self {
        Self {
            tile,
            blink_duration,
            time_bank: 0.0,
            is_on: true,
        }
    }

    pub fn tick(&mut self, dt: f32) {
        self.time_bank += dt;
        if self.time_bank >= self.blink_duration {
            self.time_bank -= self.blink_duration;
            self.is_on = !self.is_on;
        }
    }

    pub fn is_on(&self) -> bool {
        self.is_on
    }

    pub fn tile(&self) -> &AtlasTile {
        &self.tile
    }
}
