use bevy_ecs::{
    prelude::{Commands, Entity, Query, With},
    resource::Resource,
    system::ResMut,
};

use crate::systems::{Frozen, GhostCollider, PlayerControlled};

#[derive(Resource, Debug, Clone, Copy)]
pub enum StartupSequence {
    /// Stage 1: Text-only stage
    /// - Player & ghosts are hidden
    /// - READY! and PLAYER ONE text are shown
    /// - Energizers do not blink
    TextOnly {
        /// Remaining ticks in this stage
        remaining_ticks: u32,
    },
    /// Stage 2: Characters visible stage
    /// - PLAYER ONE text is hidden, READY! text remains
    /// - Ghosts and Pac-Man are now shown
    CharactersVisible {
        /// Remaining ticks in this stage
        remaining_ticks: u32,
    },
    /// Stage 3: Game begins
    /// - Final state, game is fully active
    GameActive,
}

impl StartupSequence {
    /// Creates a new StartupSequence with the specified duration in ticks
    pub fn new(text_only_ticks: u32, _characters_visible_ticks: u32) -> Self {
        Self::TextOnly {
            remaining_ticks: text_only_ticks,
        }
    }

    /// Returns true if the timer is still active (not in GameActive state)
    pub fn is_active(&self) -> bool {
        !matches!(self, StartupSequence::GameActive)
    }

    /// Returns true if we're in the game active stage
    pub fn is_game_active(&self) -> bool {
        matches!(self, StartupSequence::GameActive)
    }

    /// Ticks the timer by one frame, returning transition information if state changes
    pub fn tick(&mut self) -> Option<(StartupSequence, StartupSequence)> {
        match self {
            StartupSequence::TextOnly { remaining_ticks } => {
                if *remaining_ticks > 0 {
                    *remaining_ticks -= 1;
                    None
                } else {
                    let from = *self;
                    *self = StartupSequence::CharactersVisible {
                        remaining_ticks: 60, // 1 second at 60 FPS
                    };
                    Some((from, *self))
                }
            }
            StartupSequence::CharactersVisible { remaining_ticks } => {
                if *remaining_ticks > 0 {
                    *remaining_ticks -= 1;
                    None
                } else {
                    let from = *self;
                    *self = StartupSequence::GameActive;
                    Some((from, *self))
                }
            }
            StartupSequence::GameActive => None,
        }
    }
}

/// Handles startup sequence transitions and component management
pub fn startup_stage_system(
    mut startup: ResMut<StartupSequence>,
    mut commands: Commands,
    mut player_query: Query<Entity, With<PlayerControlled>>,
    mut ghost_query: Query<Entity, With<GhostCollider>>,
) {
    if let Some((from, to)) = startup.tick() {
        match (from, to) {
            (StartupSequence::TextOnly { .. }, StartupSequence::CharactersVisible { .. }) => {
                // TODO: Add TextOnly tag component to hide entities
                // TODO: Add CharactersVisible tag component to show entities
                // TODO: Remove TextOnly tag component
            }
            (StartupSequence::CharactersVisible { .. }, StartupSequence::GameActive) => {
                // Remove Frozen tag from all entities
                for entity in player_query.iter_mut().chain(ghost_query.iter_mut()) {
                    commands.entity(entity).remove::<Frozen>();
                }
                // TODO: Add GameActive tag component
                // TODO: Remove CharactersVisible tag component
            }
            _ => {}
        }
    }
}
