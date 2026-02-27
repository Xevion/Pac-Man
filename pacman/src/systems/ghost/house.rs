//! Ghost house exit logic managing dot counters and timer-based releases.

use super::GhostType;
use bevy_ecs::resource::Resource;

/// Tracks ghost house exit conditions
#[derive(Resource, Debug)]
pub struct GhostHouseController {
    /// Personal dot counters for each ghost (only active one at a time)
    /// [Pinky, Inky, Clyde] - Blinky starts outside
    dot_counters: [u32; 3],

    /// Which ghost's counter is currently active (index into dot_counters)
    /// 0 = Pinky, 1 = Inky, 2 = Clyde
    active_counter: Option<usize>,

    /// Global dot counter (used after death)
    global_counter: Option<u32>,

    /// Timer since last dot eaten (in ticks)
    no_dot_timer: u32,

    /// Timer limit based on level (4s for 1-4, 3s for 5+)
    timer_limit: u32,

    /// Current level
    level: u8,
}

impl Default for GhostHouseController {
    fn default() -> Self {
        Self::new(1)
    }
}

impl GhostHouseController {
    pub fn new(level: u8) -> Self {
        Self {
            dot_counters: [0; 3],
            active_counter: Some(0), // Start with Pinky
            global_counter: None,
            no_dot_timer: 0,
            timer_limit: if level < 5 { 4 * 60 } else { 3 * 60 },
            level,
        }
    }

    /// Called when Pac-Man eats a dot
    pub fn on_dot_eaten(&mut self) {
        self.no_dot_timer = 0;

        if let Some(global) = &mut self.global_counter {
            *global += 1;
        } else if let Some(idx) = self.active_counter {
            self.dot_counters[idx] += 1;
        }
    }

    /// Called when Pac-Man dies -- switches to global counter mode
    #[allow(dead_code)] // Called once death handling resets ghost house state
    pub fn on_player_death(&mut self) {
        self.global_counter = Some(0);
        self.active_counter = None;
    }

    /// Check if a ghost should exit the house
    /// Returns true if the ghost should start exiting
    pub fn should_exit(&self, ghost_type: GhostType) -> bool {
        match ghost_type {
            GhostType::Blinky => true, // Always outside
            GhostType::Pinky => self.check_pinky_exit(),
            GhostType::Inky => self.check_inky_exit(),
            GhostType::Clyde => self.check_clyde_exit(),
        }
    }

    fn check_pinky_exit(&self) -> bool {
        if let Some(global) = self.global_counter {
            global >= 7
        } else {
            self.dot_counters[0] >= self.get_dot_limit(GhostType::Pinky)
        }
    }

    fn check_inky_exit(&self) -> bool {
        if let Some(global) = self.global_counter {
            global >= 17
        } else if self.active_counter == Some(1) {
            self.dot_counters[1] >= self.get_dot_limit(GhostType::Inky)
        } else {
            false
        }
    }

    fn check_clyde_exit(&self) -> bool {
        if let Some(global) = self.global_counter {
            global >= 32
        } else if self.active_counter == Some(2) {
            self.dot_counters[2] >= self.get_dot_limit(GhostType::Clyde)
        } else {
            false
        }
    }

    fn get_dot_limit(&self, ghost_type: GhostType) -> u32 {
        match ghost_type {
            GhostType::Blinky => 0,
            GhostType::Pinky => 0,
            GhostType::Inky => {
                if self.level == 1 {
                    30
                } else {
                    0
                }
            }
            GhostType::Clyde => match self.level {
                1 => 60,
                2 => 50,
                _ => 0,
            },
        }
    }

    /// Called when a ghost exits - advance to next counter
    pub fn on_ghost_exit(&mut self, ghost_type: GhostType) {
        if self.global_counter.is_some() {
            // Deactivate global counter when Clyde exits
            if ghost_type == GhostType::Clyde && self.global_counter.is_some_and(|c| c >= 32) {
                self.global_counter = None;
                self.active_counter = None; // All ghosts out
            }
        } else {
            // Advance personal counter
            self.active_counter = match ghost_type {
                GhostType::Pinky => Some(1),              // Pinky -> Inky
                GhostType::Inky => Some(2),               // Inky -> Clyde
                GhostType::Clyde => None,                 // Clyde -> done
                GhostType::Blinky => self.active_counter, // No change
            };
        }
    }

    /// Tick the no-dot timer.
    /// Returns the index of the preferred ghost to force out (Pinky=0, Inky=1, Clyde=2), if any
    pub fn tick(&mut self) -> Option<usize> {
        self.no_dot_timer += 1;

        if self.no_dot_timer >= self.timer_limit {
            self.no_dot_timer = 0;
            // Force out the preferred ghost (active counter indicates priority)
            self.active_counter
        } else {
            None
        }
    }

    /// Reset for new level
    #[allow(dead_code)] // Called once level progression is wired up
    pub fn reset(&mut self, level: u8) {
        *self = Self::new(level);
    }
}
