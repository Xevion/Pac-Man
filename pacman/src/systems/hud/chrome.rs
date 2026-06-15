//! Window-space HUD chrome: the 1UP/score/high-score readout, remaining lives, a
//! level counter, the collected-fruit history, and a leaderboard excerpt. Drawn
//! directly onto the window at positions resolved from the active [`Layout`].
//!
//! Labels render white and their values yellow. In landscape the score/high-score/
//! lives/level groups stack at the left panel's vertical center with blank-line gaps
//! between groups, while the right panel centers a right-aligned leaderboard excerpt
//! above the fruit row. In portrait the top band lays the score block out
//! horizontally (1UP left, HIGH SCORE center, LV right) so it fits the short band,
//! and the bottom band carries lives (left) and fruit (right). Everything scales with
//! the maze's integer scale.

use bevy_ecs::event::EventWriter;
use bevy_ecs::system::{NonSendMut, Res};
use glam::UVec2;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

use crate::error::{GameError, TextureError};
use crate::map::direction::Direction;
use crate::systems::common::ScoreResource;
use crate::systems::ghost::GhostModeController;
use crate::systems::hud::{FruitSprites, LeaderboardData};
use crate::systems::layout::{Layout, Orientation};
use crate::systems::render::{CanvasResource, RenderDirty};
use crate::systems::state::PlayerLives;
use crate::texture::sprite::{AtlasTile, SpriteAtlas};
use crate::texture::sprites::{GameSprite, PacmanSprite};
use crate::texture::text::TextTexture;

/// Most recent collected fruits to show in the fruit history.
const MAX_FRUIT_ICONS: usize = 6;
/// Leaderboard rows shown in the right-panel excerpt.
const MAX_LEADERBOARD_ROWS: usize = 5;

/// Color for HUD labels ("1UP", "HIGH SCORE", "LV", "HIGH SCORES").
const LABEL: Color = Color::WHITE;
/// Color for HUD values: own/high scores, level number, and leaderboard scores.
const VALUE: Color = Color::RGB(255, 255, 0);
/// Color for leaderboard player names, kept distinct from the white header and the
/// yellow scores beside them.
const NAME: Color = Color::RGB(0, 222, 222);

/// Draws one line of HUD text at a window pixel position, reporting failures.
#[allow(clippy::too_many_arguments)]
fn draw_text(
    text: &mut TextTexture,
    canvas: &mut Canvas<Window>,
    atlas: &mut SpriteAtlas,
    errors: &mut EventWriter<GameError>,
    txt: &str,
    x: i32,
    y: i32,
    color: Color,
) {
    let pos = UVec2::new(x.max(0) as u32, y.max(0) as u32);
    if let Err(e) = text.render_with_color(canvas, atlas, txt, pos, color) {
        errors.write(TextureError::RenderFailed(format!("chrome text: {e}")).into());
    }
}

/// Draws a row of remaining-life icons left-to-right from `x`. The in-play life is
/// excluded from the count.
#[allow(clippy::too_many_arguments)]
fn draw_lives_row(
    icon: &AtlasTile,
    canvas: &mut Canvas<Window>,
    atlas: &mut SpriteAtlas,
    errors: &mut EventWriter<GameError>,
    lives: &PlayerLives,
    x: i32,
    y: i32,
    s: f32,
    gap: i32,
) {
    let w = (icon.size.x as f32 * s) as u32;
    let h = (icon.size.y as f32 * s) as u32;
    let step = w as i32 + gap;
    let displayed = lives.remaining().saturating_sub(1);
    for i in 0..displayed as i32 {
        if let Err(e) = icon.render(canvas, atlas, Rect::new(x + i * step, y, w, h)) {
            errors.write(TextureError::RenderFailed(format!("chrome life: {e}")).into());
        }
    }
}

/// Draws the collected-fruit history (most recent first). When `grow_left`, icons
/// march leftward from `anchor_x` as a right edge; otherwise rightward from it.
#[allow(clippy::too_many_arguments)]
fn draw_fruit_row(
    fruits: &FruitSprites,
    canvas: &mut Canvas<Window>,
    atlas: &mut SpriteAtlas,
    errors: &mut EventWriter<GameError>,
    anchor_x: i32,
    y: i32,
    s: f32,
    gap: i32,
    grow_left: bool,
) {
    for (i, fruit) in fruits.0.iter().rev().take(MAX_FRUIT_ICONS).enumerate() {
        let tile = match atlas.get_tile(&GameSprite::Fruit(*fruit).to_path()) {
            Ok(t) => t,
            Err(e) => {
                errors.write(TextureError::RenderFailed(format!("chrome fruit: {e}")).into());
                continue;
            }
        };
        let w = (tile.size.x as f32 * s) as u32;
        let h = (tile.size.y as f32 * s) as u32;
        let step = w as i32 + gap;
        let x = if grow_left {
            anchor_x - (i as i32 + 1) * step
        } else {
            anchor_x + i as i32 * step
        };
        if let Err(e) = tile.render(canvas, atlas, Rect::new(x, y, w, h)) {
            errors.write(TextureError::RenderFailed(format!("chrome fruit: {e}")).into());
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn chrome_render_system(
    mut canvas: NonSendMut<CanvasResource>,
    mut atlas: NonSendMut<SpriteAtlas>,
    layout: Res<Layout>,
    dirty: Res<RenderDirty>,
    score: Res<ScoreResource>,
    lives: Res<PlayerLives>,
    fruits: Res<FruitSprites>,
    leaderboard: Res<LeaderboardData>,
    mode: Res<GhostModeController>,
    mut errors: EventWriter<GameError>,
) {
    if !dirty.0 {
        return;
    }

    let canvas = &mut canvas.0;
    let atlas = &mut *atlas;
    let scale = layout.scale;
    let s = scale as f32;
    let pad = (3 * scale) as i32;
    let gap = (2 * scale) as i32;

    let mut text = TextTexture::new(s);
    let line = text.text_height() as i32 + gap;
    // A blank line separates HUD groups so the readouts read as distinct blocks.
    let group_gap = line;

    let value = format!("{:02}", score.value());
    let level = format!("{:02}", mode.level);

    // The life icon also sizes the lives/fruit rows used in cluster-height math.
    let life_icon = atlas
        .get_tile(&GameSprite::Pacman(PacmanSprite::Moving(Direction::Left, 1)).to_path())
        .ok();
    let icon_h = life_icon.as_ref().map(|t| (t.size.y as f32 * s) as i32).unwrap_or(line);

    match layout.orientation {
        Orientation::Landscape => {
            let left = layout.left.expect("landscape layout has a left panel");
            let right = layout.right.expect("landscape layout has a right panel");

            // Left column: 1UP/score, HIGH SCORE/score, lives, LV/level -- each a
            // two-line (or icon) group separated by a blank line, centered vertically.
            let cluster_h = 6 * line + icon_h + 3 * group_gap;
            let lx = left.x() + pad;
            let mut ly = left.y() + (left.height() as i32 - cluster_h).max(0) / 2;

            draw_text(&mut text, canvas, atlas, &mut errors, "1UP", lx, ly, LABEL);
            draw_text(&mut text, canvas, atlas, &mut errors, &value, lx, ly + line, VALUE);
            ly += 2 * line + group_gap;

            draw_text(&mut text, canvas, atlas, &mut errors, "HIGH SCORE", lx, ly, LABEL);
            draw_text(&mut text, canvas, atlas, &mut errors, &value, lx, ly + line, VALUE);
            ly += 2 * line + group_gap;

            if let Some(icon) = &life_icon {
                draw_lives_row(icon, canvas, atlas, &mut errors, &lives, lx, ly, s, gap);
            }
            ly += icon_h + group_gap;

            draw_text(&mut text, canvas, atlas, &mut errors, "LV", lx, ly, LABEL);
            draw_text(&mut text, canvas, atlas, &mut errors, &level, lx, ly + line, VALUE);

            // Right column: a right-aligned leaderboard excerpt above the fruit row,
            // centered vertically and flush to the panel's right margin.
            let rows = leaderboard.0.len().min(MAX_LEADERBOARD_ROWS) as i32;
            let cluster_h = line + rows * line + group_gap + icon_h;
            let right_edge = right.x() + right.width() as i32 - pad;
            let mut ry = right.y() + (right.height() as i32 - cluster_h).max(0) / 2;

            let header = "HIGH SCORES";
            let hx = right_edge - text.text_width(header) as i32;
            draw_text(&mut text, canvas, atlas, &mut errors, header, hx, ry, LABEL);
            ry += line;
            // Square font: horizontal advance per glyph equals its height.
            let char_w = text.text_height() as i32;
            for entry in leaderboard.0.iter().take(MAX_LEADERBOARD_ROWS) {
                let score = format!("{:>7}", entry.score);
                let row = format!("{} {}", entry.name, score);
                let rx = right_edge - text.text_width(&row) as i32;
                // Right-aligned as a block, but name and score are colored separately.
                draw_text(&mut text, canvas, atlas, &mut errors, entry.name, rx, ry, NAME);
                let score_x = rx + (entry.name.chars().count() as i32 + 1) * char_w;
                draw_text(&mut text, canvas, atlas, &mut errors, &score, score_x, ry, VALUE);
                ry += line;
            }
            ry += group_gap;
            draw_fruit_row(&fruits, canvas, atlas, &mut errors, right_edge, ry, s, gap, true);
        }
        Orientation::Portrait => {
            let top = layout.top.expect("portrait layout has a top band");
            let bottom = layout.bottom.expect("portrait layout has a bottom band");
            let width = top.width() as i32;

            // The short top band can't stack four lines, so the score block lays out
            // horizontally: 1UP left, HIGH SCORE centered, LV right -- each label over
            // its value, the pair centered in the band.
            let ty = top.y() + (top.height() as i32 - 2 * line).max(0) / 2;

            let lx = top.x() + pad;
            draw_text(&mut text, canvas, atlas, &mut errors, "1UP", lx, ty, LABEL);
            draw_text(&mut text, canvas, atlas, &mut errors, &value, lx, ty + line, VALUE);

            let cx = top.x() + (width - text.text_width("HIGH SCORE") as i32) / 2;
            draw_text(&mut text, canvas, atlas, &mut errors, "HIGH SCORE", cx, ty, LABEL);
            let cvx = top.x() + (width - text.text_width(&value) as i32) / 2;
            draw_text(&mut text, canvas, atlas, &mut errors, &value, cvx, ty + line, VALUE);

            let right_edge = top.x() + width - pad;
            let lvx = right_edge - text.text_width("LV") as i32;
            draw_text(&mut text, canvas, atlas, &mut errors, "LV", lvx, ty, LABEL);
            let lvvx = right_edge - text.text_width(&level) as i32;
            draw_text(&mut text, canvas, atlas, &mut errors, &level, lvvx, ty + line, VALUE);

            // Bottom band: lives at the left, fruit history at the right.
            let by = bottom.y() + (bottom.height() as i32 - icon_h).max(0) / 2;
            if let Some(icon) = &life_icon {
                draw_lives_row(icon, canvas, atlas, &mut errors, &lives, bottom.x() + pad, by, s, gap);
            }
            let fruit_anchor = bottom.x() + bottom.width() as i32 - pad;
            draw_fruit_row(&fruits, canvas, atlas, &mut errors, fruit_anchor, by, s, gap, true);
        }
    }
}
