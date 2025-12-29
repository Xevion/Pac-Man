use crate::constants;
use crate::error::{GameError, TextureError};
use crate::systems::{BackbufferResource, GameStage, PauseState, ScoreResource, StartupSequence};
use crate::texture::sprite::SpriteAtlas;
use crate::texture::text::TextTexture;
use bevy_ecs::event::EventWriter;
use bevy_ecs::system::{NonSendMut, Res};
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

/// Renders the HUD (score, lives, etc.) on top of the game.
#[allow(clippy::too_many_arguments)]
pub fn hud_render_system(
    mut backbuffer: NonSendMut<BackbufferResource>,
    mut canvas: NonSendMut<&mut Canvas<Window>>,
    mut atlas: NonSendMut<SpriteAtlas>,
    score: Res<ScoreResource>,
    stage: Res<GameStage>,
    pause_state: Res<PauseState>,
    mut errors: EventWriter<GameError>,
) {
    let _ = canvas.with_texture_canvas(&mut backbuffer.0, |canvas| {
        let mut text_renderer = TextTexture::new(1.0);

        // Render lives and high score text in white
        let lives_text = "1UP   HIGH SCORE   ";
        let lives_position = glam::UVec2::new(4 + 8 * 3, 2); // x_offset + lives_offset * 8, y_offset

        if let Err(e) = text_renderer.render(canvas, &mut atlas, lives_text, lives_position) {
            errors.write(TextureError::RenderFailed(format!("Failed to render lives text: {}", e)).into());
        }

        // Render score text
        let score_text = format!("{:02}", score.0);
        let score_offset = 7 - (score_text.len() as i32);
        let score_position = glam::UVec2::new(4 + 8 * score_offset as u32, 10); // x_offset + score_offset * 8, 8 + y_offset

        if let Err(e) = text_renderer.render(canvas, &mut atlas, &score_text, score_position) {
            errors.write(TextureError::RenderFailed(format!("Failed to render score text: {}", e)).into());
        }

        // Render high score text
        let high_score_text = format!("{:02}", score.0);
        let high_score_offset = 17 - (high_score_text.len() as i32);
        let high_score_position = glam::UVec2::new(4 + 8 * high_score_offset as u32, 10); // x_offset + score_offset * 8, 8 + y_offset
        if let Err(e) = text_renderer.render(canvas, &mut atlas, &high_score_text, high_score_position) {
            errors.write(TextureError::RenderFailed(format!("Failed to render high score text: {}", e)).into());
        }

        // Render GAME OVER text
        if matches!(*stage, GameStage::GameOver) {
            let game_over_text = "GAME  OVER";
            let game_over_width = text_renderer.text_width(game_over_text);
            let game_over_position = glam::UVec2::new((constants::CANVAS_SIZE.x - game_over_width) / 2, 160);
            if let Err(e) = text_renderer.render_with_color(canvas, &mut atlas, game_over_text, game_over_position, Color::RED) {
                errors.write(TextureError::RenderFailed(format!("Failed to render GAME OVER text: {}", e)).into());
            }
        }

        // Render text based on StartupSequence stage
        if matches!(
            *stage,
            GameStage::Starting(StartupSequence::TextOnly { .. })
                | GameStage::Starting(StartupSequence::CharactersVisible { .. })
        ) {
            let ready_text = "READY!";
            let ready_width = text_renderer.text_width(ready_text);
            let ready_position = glam::UVec2::new((constants::CANVAS_SIZE.x - ready_width) / 2, 160);
            if let Err(e) = text_renderer.render_with_color(canvas, &mut atlas, ready_text, ready_position, Color::YELLOW) {
                errors.write(TextureError::RenderFailed(format!("Failed to render READY text: {}", e)).into());
            }

            if matches!(*stage, GameStage::Starting(StartupSequence::TextOnly { .. })) {
                let player_one_text = "PLAYER ONE";
                let player_one_width = text_renderer.text_width(player_one_text);
                let player_one_position = glam::UVec2::new((constants::CANVAS_SIZE.x - player_one_width) / 2, 113);

                if let Err(e) =
                    text_renderer.render_with_color(canvas, &mut atlas, player_one_text, player_one_position, Color::CYAN)
                {
                    errors.write(TextureError::RenderFailed(format!("Failed to render PLAYER ONE text: {}", e)).into());
                }
            }
        }

        // Render pause overlay when game is paused (allowed during any stage)
        if pause_state.active() {
            // Enable blending for transparency
            canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
            
            // Draw semi-transparent black overlay
            canvas.set_draw_color(Color::RGBA(0, 0, 0, 160));
            let _ = canvas.fill_rect(Rect::new(0, 0, constants::CANVAS_SIZE.x, constants::CANVAS_SIZE.y));

            // Render "PAUSED" text centered and larger (2.5x scale)
            let mut paused_renderer = TextTexture::new(2.5);
            let paused_text = "PAUSED";
            let paused_width = paused_renderer.text_width(paused_text);
            let paused_height = paused_renderer.text_height();
            let paused_position = glam::UVec2::new(
                (constants::CANVAS_SIZE.x - paused_width) / 2,
                (constants::CANVAS_SIZE.y - paused_height) / 2
            );
            if let Err(e) = paused_renderer.render_with_color(canvas, &mut atlas, paused_text, paused_position, Color::YELLOW) {
                errors.write(TextureError::RenderFailed(format!("Failed to render PAUSED text: {}", e)).into());
            }
        }
    });
}
