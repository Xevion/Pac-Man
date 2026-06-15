//! Maze-overlay text drawn into the playfield texture in maze-local coordinates:
//! READY!, PLAYER ONE, GAME OVER, and the pause dimmer. These belong to the
//! playfield (not the HUD chrome), so they stay centered over the maze in any
//! orientation. Persistent score/lives/fruit live in [`crate::systems::hud::chrome`].

use crate::error::{GameError, TextureError};
use crate::systems::layout::PLAYFIELD_SIZE;
use crate::systems::render::{BackbufferResource, CanvasResource, RenderDirty};
use crate::systems::state::{GameStage, PauseState, StartupSequence};
use crate::texture::sprite::SpriteAtlas;
use crate::texture::text::TextTexture;
use bevy_ecs::event::EventWriter;
use bevy_ecs::system::{NonSendMut, Res};
use sdl2::pixels::Color;
use sdl2::rect::Rect;

/// Maze-local Y of the central message line (READY! / GAME OVER), matching the
/// arcade text row just below the maze center.
const MESSAGE_LINE_Y: u32 = 136;
/// Maze-local Y of the "PLAYER ONE" line shown above the message line at startup.
const PLAYER_LINE_Y: u32 = 89;

/// Renders gameplay overlay text centered over the maze playfield.
pub fn hud_overlay_system(
    mut backbuffer: NonSendMut<BackbufferResource>,
    mut canvas: NonSendMut<CanvasResource>,
    mut atlas: NonSendMut<SpriteAtlas>,
    stage: Res<GameStage>,
    pause_state: Res<PauseState>,
    dirty: Res<RenderDirty>,
    mut errors: EventWriter<GameError>,
) {
    if !dirty.0 {
        return;
    }

    let _ = canvas.with_texture_canvas(&mut backbuffer.0, |canvas| {
        let mut text = TextTexture::new(1.0);
        let centered_x = |w: u32| (PLAYFIELD_SIZE.x.saturating_sub(w)) / 2;

        if matches!(*stage, GameStage::GameOver) {
            let t = "GAME  OVER";
            let pos = glam::UVec2::new(centered_x(text.text_width(t)), MESSAGE_LINE_Y);
            if let Err(e) = text.render_with_color(canvas, &mut atlas, t, pos, Color::RED) {
                errors.write(TextureError::RenderFailed(format!("GAME OVER text: {e}")).into());
            }
        }

        if matches!(
            *stage,
            GameStage::Starting(StartupSequence::TextOnly { .. })
                | GameStage::Starting(StartupSequence::CharactersVisible { .. })
        ) {
            let t = "READY!";
            let pos = glam::UVec2::new(centered_x(text.text_width(t)), MESSAGE_LINE_Y);
            if let Err(e) = text.render_with_color(canvas, &mut atlas, t, pos, Color::YELLOW) {
                errors.write(TextureError::RenderFailed(format!("READY text: {e}")).into());
            }

            if matches!(*stage, GameStage::Starting(StartupSequence::TextOnly { .. })) {
                let t = "PLAYER ONE";
                let pos = glam::UVec2::new(centered_x(text.text_width(t)), PLAYER_LINE_Y);
                if let Err(e) = text.render_with_color(canvas, &mut atlas, t, pos, Color::CYAN) {
                    errors.write(TextureError::RenderFailed(format!("PLAYER ONE text: {e}")).into());
                }
            }
        }

        if pause_state.active() {
            canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
            canvas.set_draw_color(Color::RGBA(0, 0, 0, 160));
            let _ = canvas.fill_rect(Rect::new(0, 0, PLAYFIELD_SIZE.x, PLAYFIELD_SIZE.y));

            let mut paused = TextTexture::new(2.5);
            let t = "PAUSED";
            let pos = glam::UVec2::new(
                PLAYFIELD_SIZE.x.saturating_sub(paused.text_width(t)) / 2,
                PLAYFIELD_SIZE.y.saturating_sub(paused.text_height()) / 2,
            );
            if let Err(e) = paused.render_with_color(canvas, &mut atlas, t, pos, Color::YELLOW) {
                errors.write(TextureError::RenderFailed(format!("PAUSED text: {e}")).into());
            }
        }
    });
}
