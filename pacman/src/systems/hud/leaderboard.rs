//! Leaderboard data feeding the right-panel HUD excerpt. The default is a stub of
//! placeholder scores; swap it for live data once the game can fetch the real
//! leaderboard, and the chrome will render whatever rows are present.

use bevy_ecs::resource::Resource;

/// One leaderboard row: an arcade-style short tag and a score.
pub struct LeaderboardEntry {
    pub name: &'static str,
    pub score: u32,
}

/// Top scores shown in the right HUD panel, highest first.
#[derive(Resource)]
pub struct LeaderboardData(pub Vec<LeaderboardEntry>);

impl Default for LeaderboardData {
    fn default() -> Self {
        Self(vec![
            LeaderboardEntry {
                name: "ACE",
                score: 312000,
            },
            LeaderboardEntry {
                name: "BOB",
                score: 215600,
            },
            LeaderboardEntry {
                name: "CAT",
                score: 188200,
            },
            LeaderboardEntry {
                name: "DOT",
                score: 95400,
            },
            LeaderboardEntry {
                name: "EVE",
                score: 47800,
            },
        ])
    }
}
