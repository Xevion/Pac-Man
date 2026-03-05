use bevy_ecs::resource::Resource;

use crate::systems::animation::{DirectionalAnimation, LinearAnimation};

#[derive(Resource, Clone)]
pub struct PlayerAnimation(pub DirectionalAnimation);

#[derive(Resource, Clone)]
pub struct PlayerDeathAnimation(pub LinearAnimation);

/// Tracks whether the beginning sound has been played for the current startup sequence
#[derive(Resource, Debug, Default, Clone, Copy)]
pub struct IntroPlayed(pub bool);

/// A resource to store the number of player lives.
#[derive(Resource, Debug)]
pub struct PlayerLives(u8);

impl PlayerLives {
    /// Returns the number of remaining lives.
    pub fn remaining(&self) -> u8 {
        self.0
    }

    /// Returns whether the player has any lives left.
    pub fn is_alive(&self) -> bool {
        self.0 > 0
    }

    /// Consumes one life (saturating at zero).
    pub fn lose_life(&mut self) {
        self.0 = self.0.saturating_sub(1);
    }
}

impl Default for PlayerLives {
    fn default() -> Self {
        Self(3)
    }
}
