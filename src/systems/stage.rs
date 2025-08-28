use bevy_ecs::{
    entity::Entity,
    query::With,
    resource::Resource,
    system::{Commands, Query, ResMut},
};
use tracing::debug;

use crate::systems::{Frozen, GhostCollider, Hidden, PlayerControlled};

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
        debug!("StartupSequence transition from {from:?} to {to:?}");
        match (from, to) {
            // (StartupSequence::TextOnly { .. }, StartupSequence::CharactersVisible { .. }) => {}
            (StartupSequence::CharactersVisible { .. }, StartupSequence::GameActive) => {
                // Remove Frozen/Hidden tags from all entities and enable player input
                for entity in player_query.iter_mut().chain(ghost_query.iter_mut()) {
                    commands.entity(entity).remove::<(Frozen, Hidden)>();
                }
            }
            _ => {}
        }
    }
}
