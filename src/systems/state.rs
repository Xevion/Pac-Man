use std::mem::discriminant;

use crate::{
    map::builder::Map,
    systems::{
        AudioEvent, Blinking, DirectionalAnimation, Dying, Eaten, Frozen, Ghost, GhostCollider, GhostState, Hidden,
        LinearAnimation, Looping, PlayerControlled, Position,
    },
};
use bevy_ecs::{
    entity::Entity,
    event::EventWriter,
    query::{With, Without},
    resource::Resource,
    system::{Commands, Query, Res, ResMut},
};

#[derive(Resource, Clone)]
pub struct PlayerAnimation(pub DirectionalAnimation);

#[derive(Resource, Clone)]
pub struct PlayerDeathAnimation(pub LinearAnimation);

/// A resource to track the overall stage of the game from a high-level perspective.
#[derive(Resource, Debug, PartialEq, Eq, Clone, Copy)]
pub enum GameStage {
    Starting(StartupSequence),
    /// The main gameplay loop is active.
    Playing,
    /// The player has died and the death sequence is in progress.
    PlayerDying(DyingSequence),
    /// The level is restarting after a death.
    LevelRestarting,
    /// The game has ended.
    GameOver,
}

/// A resource that manages the multi-stage startup sequence of the game.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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
}

impl Default for GameStage {
    fn default() -> Self {
        Self::Playing
    }
}

/// The state machine for the multi-stage death sequence.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DyingSequence {
    /// Initial stage: entities are frozen, waiting for a delay.
    Frozen { remaining_ticks: u32 },
    /// Second stage: Pac-Man's death animation is playing.
    Animating { remaining_ticks: u32 },
    /// Third stage: Pac-Man is now gone, waiting a moment before the level restarts.
    Hidden { remaining_ticks: u32 },
}

/// A resource to store the number of player lives.
#[derive(Resource, Debug)]
pub struct PlayerLives(pub u8);

impl Default for PlayerLives {
    fn default() -> Self {
        Self(3)
    }
}

/// Handles startup sequence transitions and component management
#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub fn stage_system(
    mut game_state: ResMut<GameStage>,
    player_death_animation: Res<PlayerDeathAnimation>,
    player_animation: Res<PlayerAnimation>,
    mut player_lives: ResMut<PlayerLives>,
    map: Res<Map>,
    mut commands: Commands,
    mut audio_events: EventWriter<AudioEvent>,
    mut blinking_query: Query<Entity, With<Blinking>>,
    mut player_query: Query<(Entity, &mut Position), With<PlayerControlled>>,
    mut ghost_query: Query<(Entity, &Ghost, &mut Position), (With<GhostCollider>, Without<PlayerControlled>)>,
) {
    let old_state = *game_state;
    let new_state: GameStage = match &mut *game_state {
        GameStage::Starting(startup) => match startup {
            StartupSequence::TextOnly { remaining_ticks } => {
                if *remaining_ticks > 0 {
                    GameStage::Starting(StartupSequence::TextOnly {
                        remaining_ticks: *remaining_ticks - 1,
                    })
                } else {
                    GameStage::Starting(StartupSequence::CharactersVisible { remaining_ticks: 60 })
                }
            }
            StartupSequence::CharactersVisible { remaining_ticks } => {
                if *remaining_ticks > 0 {
                    GameStage::Starting(StartupSequence::CharactersVisible {
                        remaining_ticks: *remaining_ticks - 1,
                    })
                } else {
                    GameStage::Playing
                }
            }
        },
        GameStage::Playing => GameStage::Playing,
        GameStage::PlayerDying(dying) => match dying {
            DyingSequence::Frozen { remaining_ticks } => {
                if *remaining_ticks > 0 {
                    GameStage::PlayerDying(DyingSequence::Frozen {
                        remaining_ticks: *remaining_ticks - 1,
                    })
                } else {
                    let death_animation = &player_death_animation.0;
                    let remaining_ticks = (death_animation.tiles.len() * death_animation.frame_duration as usize) as u32;
                    GameStage::PlayerDying(DyingSequence::Animating { remaining_ticks })
                }
            }
            DyingSequence::Animating { remaining_ticks } => {
                if *remaining_ticks > 0 {
                    GameStage::PlayerDying(DyingSequence::Animating {
                        remaining_ticks: *remaining_ticks - 1,
                    })
                } else {
                    GameStage::PlayerDying(DyingSequence::Hidden { remaining_ticks: 60 })
                }
            }
            DyingSequence::Hidden { remaining_ticks } => {
                if *remaining_ticks > 0 {
                    GameStage::PlayerDying(DyingSequence::Hidden {
                        remaining_ticks: *remaining_ticks - 1,
                    })
                } else {
                    player_lives.0 = player_lives.0.saturating_sub(1);

                    if player_lives.0 > 0 {
                        GameStage::LevelRestarting
                    } else {
                        GameStage::GameOver
                    }
                }
            }
        },
        GameStage::LevelRestarting => GameStage::Starting(StartupSequence::CharactersVisible { remaining_ticks: 60 }),
        GameStage::GameOver => GameStage::GameOver,
    };

    if old_state == new_state {
        return;
    }

    match (old_state, new_state) {
        (GameStage::Playing, GameStage::PlayerDying(DyingSequence::Frozen { .. })) => {
            // Freeze the player & ghosts
            for entity in player_query
                .iter_mut()
                .map(|(e, _)| e)
                .chain(ghost_query.iter_mut().map(|(e, _, _)| e))
            {
                commands.entity(entity).insert(Frozen);
            }
        }
        (GameStage::PlayerDying(DyingSequence::Frozen { .. }), GameStage::PlayerDying(DyingSequence::Animating { .. })) => {
            // Hide the ghosts
            for (entity, _, _) in ghost_query.iter_mut() {
                commands.entity(entity).insert(Hidden);
            }

            // Start Pac-Man's death animation
            if let Ok((player_entity, _)) = player_query.single_mut() {
                commands
                    .entity(player_entity)
                    .insert((Dying, player_death_animation.0.clone()));
            }

            // Play the death sound
            audio_events.write(AudioEvent::PlayDeath);
        }
        (GameStage::PlayerDying(DyingSequence::Animating { .. }), GameStage::PlayerDying(DyingSequence::Hidden { .. })) => {
            // Hide the player
            if let Ok((player_entity, _)) = player_query.single_mut() {
                commands.entity(player_entity).insert(Hidden);
            }
        }
        (_, GameStage::LevelRestarting) => {
            if let Ok((player_entity, mut pos)) = player_query.single_mut() {
                *pos = Position::Stopped {
                    node: map.start_positions.pacman,
                };

                // Freeze the blinking, force them to be visible (if they were hidden by blinking)
                for entity in blinking_query.iter_mut() {
                    commands.entity(entity).insert(Frozen).remove::<Hidden>();
                }

                // Reset the player animation
                commands
                    .entity(player_entity)
                    .remove::<(Frozen, Dying, Hidden, LinearAnimation, Looping)>()
                    .insert(player_animation.0.clone());
            }

            // Reset ghost positions and state
            for (ghost_entity, ghost, mut ghost_pos) in ghost_query.iter_mut() {
                *ghost_pos = Position::Stopped {
                    node: match ghost {
                        Ghost::Blinky => map.start_positions.blinky,
                        Ghost::Pinky => map.start_positions.pinky,
                        Ghost::Inky => map.start_positions.inky,
                        Ghost::Clyde => map.start_positions.clyde,
                    },
                };
                commands
                    .entity(ghost_entity)
                    .remove::<(Frozen, Hidden, Eaten)>()
                    .insert(GhostState::Normal);
            }
        }
        (
            GameStage::Starting(StartupSequence::TextOnly { .. }),
            GameStage::Starting(StartupSequence::CharactersVisible { .. }),
        ) => {
            // Unhide the player & ghosts
            for entity in player_query
                .iter_mut()
                .map(|(e, _)| e)
                .chain(ghost_query.iter_mut().map(|(e, _, _)| e))
            {
                commands.entity(entity).remove::<Hidden>();
            }
        }
        (GameStage::Starting(StartupSequence::CharactersVisible { .. }), GameStage::Playing) => {
            // Unfreeze the player & ghosts & blinking
            for entity in player_query
                .iter_mut()
                .map(|(e, _)| e)
                .chain(ghost_query.iter_mut().map(|(e, _, _)| e))
                .chain(blinking_query.iter_mut())
            {
                commands.entity(entity).remove::<Frozen>();
            }
        }
        (GameStage::PlayerDying(..), GameStage::GameOver) => {
            // Freeze blinking
            for entity in blinking_query.iter_mut() {
                commands.entity(entity).insert(Frozen);
            }
        }
        _ => {
            let different = discriminant(&old_state) != discriminant(&new_state);
            if different {
                tracing::warn!(
                    new_state = ?new_state,
                    old_state = ?old_state,
                    "Unhandled game stage transition");
            }
        }
    }

    *game_state = new_state;
}

// if let GameState::LevelRestarting = &*game_state {
//     // When restarting, jump straight to the CharactersVisible stage
//     // and unhide the entities.
//     *startup = StartupSequence::new(0, 60 * 2); // 2 seconds for READY! text
//     if let StartupSequence::TextOnly { .. } = *startup {
//         // This will immediately transition to CharactersVisible on the next line
//     } else {
//         // Should be unreachable as we just set it
//     }

//     // Freeze Pac-Man and ghosts
//     for entity in player_query.iter().chain(ghost_query.iter()) {
//         commands.entity(entity).insert(Frozen);
//     }

//     *game_state = GameState::Playing;
// }

// if let Some((old_state, new_state)) = startup.tick() {
//     debug!("StartupSequence transition from {old_state:?} to {new_state:?}");
//     match (old_state, new_state) {
//         (StartupSequence::TextOnly { .. }, StartupSequence::CharactersVisible { .. }) => {
//             // Unhide the player & ghosts
//             for entity in player_query.iter().chain(ghost_query.iter()) {
//                 commands.entity(entity).remove::<Hidden>();
//             }
//         }
//         (StartupSequence::CharactersVisible { .. }, StartupSequence::GameActive) => {
//             // Unfreeze Pac-Man, ghosts and energizers
//             for entity in player_query.iter().chain(ghost_query.iter()).chain(blinking_query.iter()) {
//                 commands.entity(entity).remove::<Frozen>();
//             }
//             *game_state = GameState::Playing;
//         }
//         _ => {}
//     }
// }
