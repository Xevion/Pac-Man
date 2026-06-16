use tracing::{debug, info};

use crate::constants;
use crate::events::StageTransition;
use crate::map::builder::Map;
use crate::map::direction::Direction;
use crate::systems::animation::{Blinking, DirectionalAnimation, Dying, LinearAnimation, Looping};
use crate::systems::audio::AudioEvent;
use crate::systems::collision::{GhostCollider, ItemCollider};
use crate::systems::common::{EntityType, Frozen};
use crate::systems::ghost::{GhostState, GhostType};
use crate::systems::item::SpawnTrigger;
use crate::systems::movement::{NodeId, Position, Velocity};
use crate::systems::player::PlayerControlled;
use crate::systems::render::Visibility;
use bevy_ecs::{
    entity::Entity,
    event::EventWriter,
    observer::Trigger,
    query::{With, Without},
    system::{Commands, Query, Res, ResMut, Single, SystemParam},
};

use super::{PlayerAnimation, PlayerDeathAnimation, Session, TooSimilar};

/// The overall stage of the game from a high-level perspective. Lives inside
/// [`crate::systems::state::Session`] as the gameplay sub-machine's state.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub enum GameStage {
    Starting(StartupSequence),
    /// The main gameplay loop is active.
    #[default]
    Playing,
    /// Short freeze after Pac-Man eats a ghost to display bonus score
    GhostEatenPause {
        remaining_ticks: u32,
        ghost_entity: Entity,
        ghost_type: GhostType,
        node: NodeId,
    },
    /// The player has died and the death sequence is in progress. At the end, the player will return to the startup sequence or game over.
    PlayerDying(DyingSequence),
    /// The game has ended.
    GameOver,
}

impl GameStage {
    /// The stage a fresh gameplay session begins in: the opening startup sequence.
    /// Waiting for a user gesture (browser autoplay policy) is now the Title scene's
    /// job, not a stage.
    pub fn initial() -> Self {
        GameStage::Starting(StartupSequence::TextOnly {
            remaining_ticks: constants::startup::STARTUP_FRAMES,
        })
    }

    /// Applies the entity side-effects of entering `self`, given the stage being left.
    ///
    /// Called by [`stage_system`] for every tick-driven transition. The match is
    /// exhaustive over the *new* stage, so adding a `GameStage` variant fails to compile
    /// until its entry effects are defined here -- the transition table is the type.
    /// `old` is consulted only for the two genuinely pair-dependent edges into `Playing`.
    ///
    /// GhostEatenPause *entry* is event-driven (a ghost is eaten, not a tick elapsing),
    /// so it lives in [`enter_ghost_eaten_pause`] and is a deliberate no-op here.
    fn on_enter(self, old: GameStage, res: &mut StageResources) {
        match self {
            GameStage::Starting(StartupSequence::TextOnly { .. }) => {
                // No entity effects: characters stay hidden, energizers stay static.
            }
            GameStage::Starting(StartupSequence::CharactersVisible { .. }) => {
                res.reveal_actors();
            }
            GameStage::Playing => match old {
                GameStage::GhostEatenPause { ghost_entity, .. } => {
                    res.unfreeze_actors();
                    res.reveal_actors();
                    // The just-eaten ghost returns as Eyes, heading home.
                    res.commands.entity(ghost_entity).insert(GhostState::Eyes);
                }
                GameStage::Starting(StartupSequence::CharactersVisible { .. }) => {
                    res.unfreeze_actors();
                    res.unfreeze_blinking();
                    // Arm the intro jingle for the next round.
                    res.session.intro_played = false;
                }
                _ => {}
            },
            GameStage::GhostEatenPause { .. } => {
                // Entry is event-driven; handled by `enter_ghost_eaten_pause`.
            }
            GameStage::PlayerDying(DyingSequence::Frozen { .. }) => {
                res.freeze_actors();
            }
            GameStage::PlayerDying(DyingSequence::Animating { .. }) => {
                res.hide_ghosts();
                res.start_player_death_animation();
            }
            GameStage::PlayerDying(DyingSequence::Hidden { .. }) => {
                // Death animation done: clear the board and return everyone to their marks,
                // hidden and frozen, ready for the next startup sequence.
                res.freeze_blinking_visible();
                res.despawn_fruit();
                res.reset_player_to_start();
                res.reset_ghosts_to_start();
            }
            GameStage::GameOver => {
                res.freeze_blinking();
            }
        }
    }
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

/// Grouped parameters for the stage system: the session sub-machine plus the entities its
/// transitions reveal, freeze, and reset. Bundling them as one `SystemParam` lets
/// [`GameStage::on_enter`] read like a transition table over a single borrowed context.
#[derive(SystemParam)]
pub struct StageResources<'w, 's> {
    pub session: ResMut<'w, Session>,
    pub player_death_animation: Res<'w, PlayerDeathAnimation>,
    pub player_animation: Res<'w, PlayerAnimation>,
    pub map: Res<'w, Map>,
    pub commands: Commands<'w, 's>,
    pub audio_events: EventWriter<'w, AudioEvent>,
    pub player: Single<'w, Entity, With<PlayerControlled>>,
    pub blinking: Query<'w, 's, Entity, With<Blinking>>,
    pub items: Query<'w, 's, (Entity, &'static EntityType), With<ItemCollider>>,
    #[allow(clippy::type_complexity)]
    pub ghosts: Query<'w, 's, (Entity, &'static GhostType), (With<GhostCollider>, Without<PlayerControlled>)>,
}

/// Named choreography the stage transitions are built from. Each method is one intent --
/// freeze, reveal, hide, reset -- so [`GameStage::on_enter`] reads as a sequence of intents
/// rather than a wall of per-entity component edits.
impl StageResources<'_, '_> {
    /// Freeze the player and every ghost, halting movement and animation.
    fn freeze_actors(&mut self) {
        let player = *self.player;
        self.commands.entity(player).insert(Frozen);
        for (entity, _) in self.ghosts.iter() {
            self.commands.entity(entity).insert(Frozen);
        }
    }

    /// Unfreeze the player and every ghost, resuming them.
    fn unfreeze_actors(&mut self) {
        let player = *self.player;
        self.commands.entity(player).remove::<Frozen>();
        for (entity, _) in self.ghosts.iter() {
            self.commands.entity(entity).remove::<Frozen>();
        }
    }

    /// Make the player and every ghost visible.
    fn reveal_actors(&mut self) {
        let player = *self.player;
        self.commands.entity(player).insert(Visibility::visible());
        for (entity, _) in self.ghosts.iter() {
            self.commands.entity(entity).insert(Visibility::visible());
        }
    }

    /// Hide every ghost (the player keeps whatever visibility it has).
    fn hide_ghosts(&mut self) {
        for (entity, _) in self.ghosts.iter() {
            self.commands.entity(entity).insert(Visibility::hidden());
        }
    }

    /// Freeze the blinking energizers, stopping their blink animation.
    fn freeze_blinking(&mut self) {
        for entity in self.blinking.iter() {
            self.commands.entity(entity).insert(Frozen);
        }
    }

    /// Unfreeze the blinking energizers, resuming the blink.
    fn unfreeze_blinking(&mut self) {
        for entity in self.blinking.iter() {
            self.commands.entity(entity).remove::<Frozen>();
        }
    }

    /// Freeze the blinking energizers and force them visible, so one caught mid-blink-hidden
    /// doesn't vanish while the board is held for a death reset.
    fn freeze_blinking_visible(&mut self) {
        for entity in self.blinking.iter() {
            self.commands.entity(entity).insert(Frozen).insert(Visibility::visible());
        }
    }

    /// Despawn any on-board fruit (bonus item) entities.
    fn despawn_fruit(&mut self) {
        let fruit: Vec<Entity> = self
            .items
            .iter()
            .filter_map(|(entity, entity_type)| matches!(entity_type, EntityType::Fruit(_)).then_some(entity))
            .collect();
        for entity in fruit {
            self.commands.entity(entity).despawn();
        }
    }

    /// Swap the player's directional animation for the death animation and play the sound.
    fn start_player_death_animation(&mut self) {
        let player = *self.player;
        self.commands
            .entity(player)
            .remove::<DirectionalAnimation>()
            .insert((Dying, self.player_death_animation.0.clone()));
        self.audio_events
            .write(AudioEvent::PlaySound(crate::audio::Sound::PacmanDeath));
    }

    /// Return the player to its start node: clear the death animation, restore the normal
    /// animation and velocity, and leave it frozen and hidden for the next startup.
    fn reset_player_to_start(&mut self) {
        let player = *self.player;
        let node = self.map.start_positions.pacman;
        self.commands
            .entity(player)
            .remove::<(Dying, LinearAnimation, Looping)>()
            .insert((
                Velocity {
                    speed: constants::mechanics::PLAYER_SPEED,
                    direction: Direction::Left,
                },
                Position::Stopped { node },
                self.player_animation.0.clone(),
                Visibility::hidden(),
                Frozen,
            ));
    }

    /// Return every ghost to its start node and initial state, frozen and hidden.
    fn reset_ghosts_to_start(&mut self) {
        let resets: Vec<(Entity, GhostType)> = self.ghosts.iter().map(|(entity, ghost)| (entity, *ghost)).collect();
        for (entity, ghost) in resets {
            let node = self.map.start_positions.ghost_start(ghost);
            self.commands.entity(entity).insert((
                ghost.initial_state(),
                Position::Stopped { node },
                Frozen,
                Visibility::hidden(),
            ));
        }
    }
}

/// Advances the gameplay sub-machine one tick and applies the side-effects of any
/// transition. The per-stage tick logic decides the next stage; [`GameStage::on_enter`]
/// then applies the freeze/hide/reset effects.
pub fn stage_system(mut res: StageResources) {
    let old_state = res.session.stage();

    let new_state: GameStage = match res.session.stage() {
        GameStage::Playing => {
            // This is the default state, do nothing
            res.session.stage()
        }
        GameStage::GhostEatenPause {
            remaining_ticks,
            ghost_entity,
            ghost_type,
            node,
        } => {
            if remaining_ticks > 0 {
                GameStage::GhostEatenPause {
                    remaining_ticks: remaining_ticks.saturating_sub(1),
                    ghost_entity,
                    ghost_type,
                    node,
                }
            } else {
                debug!("Ghost eaten pause ended, resuming gameplay");
                GameStage::Playing
            }
        }
        GameStage::Starting(sequence) => match sequence {
            StartupSequence::TextOnly { remaining_ticks } => {
                // Play the beginning sound once at the start of TextOnly stage
                if !res.session.intro_played {
                    res.audio_events.write(AudioEvent::PlaySound(crate::audio::Sound::Beginning));
                    res.session.intro_played = true;
                }
                if remaining_ticks > 0 {
                    GameStage::Starting(StartupSequence::TextOnly {
                        remaining_ticks: remaining_ticks.saturating_sub(1),
                    })
                } else {
                    GameStage::Starting(StartupSequence::CharactersVisible {
                        remaining_ticks: constants::startup::CHARACTERS_VISIBLE_TICKS,
                    })
                }
            }
            StartupSequence::CharactersVisible { remaining_ticks } => {
                if remaining_ticks > 0 {
                    GameStage::Starting(StartupSequence::CharactersVisible {
                        remaining_ticks: remaining_ticks.saturating_sub(1),
                    })
                } else {
                    info!("Startup sequence completed, beginning gameplay");
                    GameStage::Playing
                }
            }
        },
        GameStage::PlayerDying(sequence) => match sequence {
            DyingSequence::Frozen { remaining_ticks } => {
                if remaining_ticks > 0 {
                    GameStage::PlayerDying(DyingSequence::Frozen {
                        remaining_ticks: remaining_ticks.saturating_sub(1),
                    })
                } else {
                    let death_animation = &res.player_death_animation.0;
                    let remaining_ticks = (death_animation.tiles.len() * death_animation.frame_duration as usize) as u32;
                    debug!(animation_frames = remaining_ticks, "Starting player death animation");
                    GameStage::PlayerDying(DyingSequence::Animating { remaining_ticks })
                }
            }
            DyingSequence::Animating { remaining_ticks } => {
                if remaining_ticks > 0 {
                    GameStage::PlayerDying(DyingSequence::Animating {
                        remaining_ticks: remaining_ticks.saturating_sub(1),
                    })
                } else {
                    GameStage::PlayerDying(DyingSequence::Hidden {
                        remaining_ticks: constants::mechanics::DEATH_HIDDEN_TICKS,
                    })
                }
            }
            DyingSequence::Hidden { remaining_ticks } => {
                if remaining_ticks > 0 {
                    GameStage::PlayerDying(DyingSequence::Hidden {
                        remaining_ticks: remaining_ticks.saturating_sub(1),
                    })
                } else {
                    res.session.lives.lose_life();

                    if res.session.lives.is_alive() {
                        info!(
                            remaining_lives = res.session.lives.remaining(),
                            "Player died, returning to startup sequence"
                        );
                        GameStage::Starting(StartupSequence::CharactersVisible {
                            remaining_ticks: constants::startup::CHARACTERS_VISIBLE_TICKS,
                        })
                    } else {
                        info!("All lives lost, game over");
                        GameStage::GameOver
                    }
                }
            }
        },
        GameStage::GameOver => res.session.stage(),
    };

    if old_state == new_state {
        return;
    }

    if !old_state.too_similar(&new_state) {
        debug!(old_state = ?old_state, new_state = ?new_state, "Game stage transition");
    }

    new_state.on_enter(old_state, &mut res);

    res.session.set_stage(new_state);
}

/// Enters the GhostEatenPause stage when Pac-Man eats a frightened ghost.
///
/// Triggered from `ghost_collision_observer`, so the freeze/hide/bonus effects run here
/// rather than through `stage_system`'s tick-driven [`GameStage::on_enter`] -- the entry
/// is a discrete event, not a tick elapsing. This replaces the old buffered
/// `StageTransition` event (and the teardown drain it forced on `despawn_gameplay`).
#[allow(clippy::type_complexity)]
pub fn enter_ghost_eaten_pause(
    trigger: Trigger<StageTransition>,
    mut session: ResMut<Session>,
    mut commands: Commands,
    player: Single<(Entity, &Position), With<PlayerControlled>>,
    ghosts: Query<(Entity, &GhostState), (With<GhostCollider>, Without<PlayerControlled>)>,
) {
    let StageTransition::GhostEatenPause {
        ghost_entity,
        ghost_type,
        value,
    } = *trigger;

    // Only enter the pause from live gameplay: if a fatal collision already flipped the
    // session into PlayerDying this frame, that takes precedence.
    if !matches!(session.stage(), GameStage::Playing) {
        return;
    }

    let player_entity = player.0;
    let node = player.1.current_node();
    debug!(ghost = ?ghost_type, node, "Ghost eaten, entering pause state");

    // Freeze the player & non-eaten ghosts.
    commands.entity(player_entity).insert(Frozen);
    commands.entity(ghost_entity).insert(Frozen);
    for (entity, state) in ghosts.iter() {
        if *state != GhostState::Eyes {
            commands.entity(entity).insert(Frozen);
        }
    }

    // Hide the player & eaten ghost.
    commands.entity(player_entity).insert(Visibility::hidden());
    commands.entity(ghost_entity).insert(Visibility::hidden());

    // Spawn the bonus-score entity at Pac-Man's node, showing the eat's actual value.
    commands.trigger(SpawnTrigger::Bonus {
        position: Position::Stopped { node },
        value,
        ttl: constants::mechanics::GHOST_EATEN_PAUSE_TICKS,
    });

    session.set_stage(GameStage::GhostEatenPause {
        remaining_ticks: constants::mechanics::GHOST_EATEN_PAUSE_TICKS,
        ghost_entity,
        ghost_type,
        node,
    });
}
