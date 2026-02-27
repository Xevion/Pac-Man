//! Blinky's special "Cruise Elroy" behavior - speed boosts based on remaining pellets.

use bevy_ecs::component::Component;

/// Elroy state - only attached to Blinky
#[derive(Component, Debug, Default)]
pub struct Elroy {
    pub stage: ElroyStage,
    /// Suspended when Clyde is in house after player death
    pub suspended: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ElroyStage {
    #[default]
    None,
    /// First speed boost
    Stage1,
    /// Second speed boost (faster than Pac-Man!)
    Stage2,
}

/// Dot thresholds for Elroy activation per level
pub fn elroy_thresholds(level: u8) -> (u32, u32) {
    match level {
        1 => (20, 10),
        2 => (30, 15),
        3..=4 => (40, 20),
        5..=6 => (40, 20),
        7..=8 => (50, 25),
        9..=11 => (60, 30),
        12..=14 => (80, 40),
        15..=17 => (100, 50),
        _ => (120, 60),
    }
}

/// Elroy speed multipliers
pub fn elroy_speed(stage: ElroyStage, level: u8) -> f32 {
    match (stage, level) {
        (ElroyStage::None, _) => 1.0,
        (ElroyStage::Stage1, 1) => 0.80,
        (ElroyStage::Stage1, 2..=4) => 0.90,
        (ElroyStage::Stage1, _) => 1.00,
        (ElroyStage::Stage2, 1) => 0.85,
        (ElroyStage::Stage2, 2..=4) => 0.95,
        (ElroyStage::Stage2, _) => 1.05, // Faster than Pac-Man!
    }
}

/// Marker component for Blinky (to easily query him for Inky's targeting)
#[derive(Component, Debug, Default)]
pub struct BlinkyMarker;
