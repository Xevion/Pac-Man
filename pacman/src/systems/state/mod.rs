mod fullscreen;
mod pause;
mod resources;
mod stage;

#[cfg(not(target_os = "emscripten"))]
pub use self::fullscreen::*;
pub use self::pause::*;
pub use self::resources::*;
pub use self::stage::*;

use std::mem::discriminant;

pub trait TooSimilar {
    fn too_similar(&self, other: &Self) -> bool;
}

impl TooSimilar for GameStage {
    fn too_similar(&self, other: &Self) -> bool {
        discriminant(self) == discriminant(other) && {
            // These states are very simple, so they're 'too similar' automatically
            #[cfg(target_os = "emscripten")]
            if matches!(
                self,
                GameStage::Playing | GameStage::GameOver | GameStage::WaitingForInteraction
            ) {
                return true;
            }
            #[cfg(not(target_os = "emscripten"))]
            if matches!(self, GameStage::Playing | GameStage::GameOver) {
                return true;
            }

            // Since the discriminant is the same but the values are different, it's the interior value that is somehow different
            match (self, other) {
                // These states are similar if their interior values are similar as well
                (GameStage::Starting(startup), GameStage::Starting(other)) => startup.too_similar(other),
                (GameStage::PlayerDying(dying), GameStage::PlayerDying(other)) => dying.too_similar(other),
                (
                    GameStage::GhostEatenPause {
                        ghost_entity,
                        ghost_type,
                        node,
                        ..
                    },
                    GameStage::GhostEatenPause {
                        ghost_entity: other_ghost_entity,
                        ghost_type: other_ghost_type,
                        node: other_node,
                        ..
                    },
                ) => ghost_entity == other_ghost_entity && ghost_type == other_ghost_type && node == other_node,
                // Already handled, but kept to properly exhaust the match
                #[cfg(target_os = "emscripten")]
                (GameStage::Playing, _) | (GameStage::GameOver, _) | (GameStage::WaitingForInteraction, _) => unreachable!(),
                #[cfg(not(target_os = "emscripten"))]
                (GameStage::Playing, _) | (GameStage::GameOver, _) => unreachable!(),
                _ => unreachable!(),
            }
        }
    }
}

impl TooSimilar for StartupSequence {
    fn too_similar(&self, other: &Self) -> bool {
        discriminant(self) == discriminant(other)
    }
}

impl TooSimilar for DyingSequence {
    fn too_similar(&self, other: &Self) -> bool {
        discriminant(self) == discriminant(other)
    }
}
