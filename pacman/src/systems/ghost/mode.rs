//! Global scatter/chase mode controller managing timing and transitions for all ghosts.

use bevy_ecs::resource::Resource;

/// Global controller managing the scatter/chase cycle for all ghosts
#[derive(Resource, Debug)]
pub struct GhostModeController {
    /// Current scatter/chase mode (Frightened handled per-ghost)
    pub mode: ScatterChaseMode,
    /// Ticks remaining in current mode phase
    pub mode_timer: u32,
    /// Index into scatter/chase timing pattern (0-7)
    pub phase_index: usize,
    /// True while any ghost is frightened - pauses the scatter/chase timer
    pub paused: bool,
    /// Current game level (affects timing)
    pub level: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScatterChaseMode {
    Scatter,
    Chase,
}

impl Default for GhostModeController {
    fn default() -> Self {
        Self::new(1)
    }
}

/// Scatter/Chase timing pattern: (scatter_ticks, chase_ticks)
/// Phase 0-1: Scatter 7s, Chase 20s
/// Phase 2-3: Scatter 7s, Chase 20s
/// Phase 4-5: Scatter 5s, Chase 20s
/// Phase 6-7: Scatter 5s, Chase indefinite
const LEVEL_1_TIMINGS: [(u32, u32); 4] = [
    (7 * 60, 20 * 60),  // Phase 0-1
    (7 * 60, 20 * 60),  // Phase 2-3
    (5 * 60, 20 * 60),  // Phase 4-5
    (5 * 60, u32::MAX), // Phase 6-7 (indefinite chase)
];

const LEVEL_2_4_TIMINGS: [(u32, u32); 4] = [
    (7 * 60, 20 * 60),
    (7 * 60, 20 * 60),
    (5 * 60, 1033 * 60),
    (1, u32::MAX), // 1/60th second scatter, then indefinite
];

const LEVEL_5_PLUS_TIMINGS: [(u32, u32); 4] = [(5 * 60, 20 * 60), (5 * 60, 20 * 60), (5 * 60, 1037 * 60), (1, u32::MAX)];

impl GhostModeController {
    pub fn new(level: u8) -> Self {
        Self {
            level,
            mode: ScatterChaseMode::Scatter,
            mode_timer: Self::get_phase_duration(0, level),
            phase_index: 0,
            paused: false,
        }
    }

    /// Get duration for a specific phase at a specific level
    fn get_phase_duration(phase_index: usize, level: u8) -> u32 {
        let timings = match level {
            1 => &LEVEL_1_TIMINGS,
            2..=4 => &LEVEL_2_4_TIMINGS,
            _ => &LEVEL_5_PLUS_TIMINGS,
        };

        let pair_index = phase_index / 2;
        let is_chase = phase_index % 2 == 1;

        if pair_index >= timings.len() {
            return u32::MAX; // Indefinite
        }

        if is_chase {
            timings[pair_index].1
        } else {
            timings[pair_index].0
        }
    }

    /// Tick the mode controller. Returns true if mode changed (ghosts should reverse)
    pub fn tick(&mut self) -> bool {
        if self.paused || self.mode_timer == u32::MAX {
            return false;
        }

        if self.mode_timer > 0 {
            self.mode_timer -= 1;
            return false;
        }

        // Time to switch modes
        self.phase_index += 1;
        self.mode = match self.mode {
            ScatterChaseMode::Scatter => ScatterChaseMode::Chase,
            ScatterChaseMode::Chase => ScatterChaseMode::Scatter,
        };
        self.mode_timer = Self::get_phase_duration(self.phase_index, self.level);

        true // Mode changed - signal reversal
    }

    /// Pause the timer (called when ghosts become frightened)
    pub fn pause(&mut self) {
        self.paused = true;
    }

    /// Resume the timer (called when no ghosts are frightened)
    pub fn resume(&mut self) {
        self.paused = false;
    }

    /// Reset for new level or after death
    #[allow(dead_code)] // Called once level progression is wired up
    pub fn reset(&mut self, level: u8) {
        self.level = level;
        self.phase_index = 0;
        self.mode = ScatterChaseMode::Scatter;
        self.mode_timer = Self::get_phase_duration(0, level);
        self.paused = false;
    }
}
