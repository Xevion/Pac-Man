//! Leaderboard data feeding the right-panel HUD excerpt. On the web build, JS pushes
//! live top scores in via the `pacman_leaderboard_clear`/`pacman_leaderboard_push` FFI
//! exports (see `main.rs`); the default below is only the pre-fetch placeholder shown
//! before that first happens.

use bevy_ecs::resource::Resource;

/// One leaderboard row: an arcade-style short tag and a score.
pub struct LeaderboardEntry {
    pub name: String,
    pub score: u32,
}

/// Top scores shown in the right HUD panel, highest first.
#[derive(Resource)]
pub struct LeaderboardData(pub Vec<LeaderboardEntry>);

impl Default for LeaderboardData {
    fn default() -> Self {
        Self(vec![
            LeaderboardEntry {
                name: "ACE".to_string(),
                score: 312000,
            },
            LeaderboardEntry {
                name: "BOB".to_string(),
                score: 215600,
            },
            LeaderboardEntry {
                name: "CAT".to_string(),
                score: 188200,
            },
            LeaderboardEntry {
                name: "DOT".to_string(),
                score: 95400,
            },
            LeaderboardEntry {
                name: "EVE".to_string(),
                score: 47800,
            },
        ])
    }
}
