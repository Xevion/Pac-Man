use rand::rngs::SmallRng;
use rand::Rng;
use rand::SeedableRng;

use crate::constants::MapTile;
use crate::constants::BOARD_CELL_SIZE;
use crate::entity::direction::Direction;
use crate::entity::pacman::Pacman;
use crate::entity::speed::SimpleTickModulator;
use crate::entity::{Entity, MovableEntity, Moving, Renderable};
use crate::map::Map;
use crate::texture::{
    animated::AnimatedTexture, blinking::BlinkingTexture, directional::DirectionalAnimatedTexture, get_atlas_tile,
    sprite::SpriteAtlas,
};
use anyhow::Result;
use glam::{IVec2, UVec2};
use sdl2::pixels::Color;
use sdl2::render::WindowCanvas;
use std::cell::RefCell;
use std::rc::Rc;

/// The different modes a ghost can be in
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GhostMode {
    /// Chase mode - ghost actively pursues Pac-Man using its unique strategy
    Chase,
    /// Scatter mode - ghost heads to its home corner
    Scatter,
    /// Frightened mode - ghost moves randomly and can be eaten
    Frightened,
    /// Eyes mode - ghost returns to the ghost house after being eaten
    Eyes,
    /// House mode - ghost is in the ghost house, waiting to exit
    House(HouseMode),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HouseMode {
    Entering,
    Exiting,
    Waiting,
}

/// The different ghost personalities
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GhostType {
    Blinky, // Red - Shadow
    Pinky,  // Pink - Speedy
    Inky,   // Cyan - Bashful
    Clyde,  // Orange - Pokey
}

impl GhostType {
    /// Returns the color of the ghost.
    pub fn color(&self) -> Color {
        match self {
            GhostType::Blinky => Color::RGB(255, 0, 0),
            GhostType::Pinky => Color::RGB(255, 184, 255),
            GhostType::Inky => Color::RGB(0, 255, 255),
            GhostType::Clyde => Color::RGB(255, 184, 82),
        }
    }
}

/// Base ghost struct that contains common functionality
pub struct Ghost {
    /// Shared movement and position fields.
    pub base: MovableEntity,
    /// The current mode of the ghost
    pub mode: GhostMode,
    /// The type/personality of this ghost
    pub ghost_type: GhostType,
    /// Reference to Pac-Man for targeting
    pub pacman: Rc<RefCell<Pacman>>,
    pub texture: DirectionalAnimatedTexture,
    pub frightened_texture: BlinkingTexture,
    pub eyes_texture: DirectionalAnimatedTexture,
    pub house_offset: i32,
    pub current_house_offset: i32,
}

impl Ghost {
    /// Creates a new ghost instance
    pub fn new(
        ghost_type: GhostType,
        starting_position: UVec2,
        atlas: Rc<RefCell<SpriteAtlas>>,
        map: Rc<RefCell<Map>>,
        pacman: Rc<RefCell<Pacman>>,
        house_offset: i32,
    ) -> Ghost {
        let pixel_position = Map::cell_to_pixel(starting_position);
        let name = match ghost_type {
            GhostType::Blinky => "blinky",
            GhostType::Pinky => "pinky",
            GhostType::Inky => "inky",
            GhostType::Clyde => "clyde",
        };
        let get = |dir: &str, suffix: &str| get_atlas_tile(&atlas, &format!("ghost/{name}/{dir}_{suffix}.png"));

        let texture = DirectionalAnimatedTexture::new(
            vec![get("up", "a"), get("up", "b")],
            vec![get("down", "a"), get("down", "b")],
            vec![get("left", "a"), get("left", "b")],
            vec![get("right", "a"), get("right", "b")],
            25,
        );

        let frightened_texture = BlinkingTexture::new(
            AnimatedTexture::new(
                vec![
                    get_atlas_tile(&atlas, "ghost/frightened/blue_a.png"),
                    get_atlas_tile(&atlas, "ghost/frightened/blue_b.png"),
                ],
                10,
            ),
            45,
            15,
        );

        let eyes_get = |dir: &str| get_atlas_tile(&atlas, &format!("ghost/eyes/{dir}.png"));

        let eyes_texture = DirectionalAnimatedTexture::new(
            vec![eyes_get("up")],
            vec![eyes_get("down")],
            vec![eyes_get("left")],
            vec![eyes_get("right")],
            0,
        );

        Ghost {
            base: MovableEntity::new(
                pixel_position,
                starting_position,
                Direction::Left,
                SimpleTickModulator::new(0.9375),
                map,
            ),
            mode: GhostMode::House(HouseMode::Waiting),
            ghost_type,
            pacman,
            texture,
            frightened_texture,
            eyes_texture,
            house_offset,
            current_house_offset: house_offset,
        }
    }

    /// Gets the target tile for this ghost based on its current mode
    pub fn get_target_tile(&self) -> Option<IVec2> {
        match self.mode {
            GhostMode::Scatter => Some(self.get_scatter_target()),
            GhostMode::Chase => Some(self.get_chase_target()),
            GhostMode::Frightened => Some(self.get_random_target()),
            GhostMode::Eyes => Some(self.get_house_target()),
            GhostMode::House(_) => None,
        }
    }

    /// Gets this ghost's home corner target for scatter mode
    fn get_scatter_target(&self) -> IVec2 {
        match self.ghost_type {
            GhostType::Blinky => IVec2::new(25, 0), // Top right
            GhostType::Pinky => IVec2::new(2, 0),   // Top left
            GhostType::Inky => IVec2::new(27, 35),  // Bottom right
            GhostType::Clyde => IVec2::new(0, 35),  // Bottom left
        }
    }

    /// Gets a random adjacent tile for frightened mode
    fn get_random_target(&self) -> IVec2 {
        let mut rng = SmallRng::from_os_rng();
        let mut possible_moves = Vec::new();

        // Check all four directions
        for dir in &[Direction::Up, Direction::Down, Direction::Left, Direction::Right] {
            // Don't allow reversing direction
            if *dir == self.base.direction.opposite() {
                continue;
            }

            let next_cell = self.base.next_cell(Some(*dir));
            if !matches!(self.base.map.borrow().get_tile(next_cell), Some(MapTile::Wall)) {
                possible_moves.push(next_cell);
            }
        }

        if possible_moves.is_empty() {
            // No valid moves, must reverse
            self.base.next_cell(Some(self.base.direction.opposite()))
        } else {
            // Choose a random valid move
            possible_moves[rng.random_range(0..possible_moves.len())]
        }
    }

    /// Gets the ghost house target for returning eyes
    fn get_house_target(&self) -> IVec2 {
        IVec2::new(13, 14) // Center of ghost house
    }

    /// Gets this ghost's chase mode target based on its personality
    fn get_chase_target(&self) -> IVec2 {
        let pacman = self.pacman.borrow();
        let pacman_cell = pacman.base().cell_position;
        let pacman_direction = pacman.base.direction;

        match self.ghost_type {
            GhostType::Blinky => {
                // Blinky (Red) - Directly targets Pac-Man's current position
                IVec2::new(pacman_cell.x as i32, pacman_cell.y as i32)
            }
            GhostType::Pinky => {
                // Pinky (Pink) - Targets 4 cells ahead of Pac-Man in his direction
                let offset = pacman_direction.offset();
                let target_x = (pacman_cell.x as i32) + (offset.x * 4);
                let target_y = (pacman_cell.y as i32) + (offset.y * 4);
                IVec2::new(target_x, target_y)
            }
            GhostType::Inky => {
                // Inky (Cyan) - Uses Blinky's position and Pac-Man's position to calculate target
                // For now, just target Pac-Man with some randomness
                let mut rng = SmallRng::from_os_rng();
                let random_offset_x = rng.random_range(-2..=2);
                let random_offset_y = rng.random_range(-2..=2);
                IVec2::new(
                    (pacman_cell.x as i32) + random_offset_x,
                    (pacman_cell.y as i32) + random_offset_y,
                )
            }
            GhostType::Clyde => {
                // Clyde (Orange) - Targets Pac-Man when far, runs to scatter corner when close
                let distance = ((self.base.base.cell_position.x as i32 - pacman_cell.x as i32).pow(2)
                    + (self.base.base.cell_position.y as i32 - pacman_cell.y as i32).pow(2))
                    as f32;
                let distance = distance.sqrt();

                if distance > 8.0 {
                    // Far from Pac-Man - chase
                    IVec2::new(pacman_cell.x as i32, pacman_cell.y as i32)
                } else {
                    // Close to Pac-Man - scatter to bottom left
                    IVec2::new(0, 35)
                }
            }
        }
    }

    /// Calculates the path to the target tile using the A* algorithm.
    pub fn get_path_to_target(&self, target: UVec2) -> Option<(Vec<UVec2>, u32)> {
        let start = self.base.base.cell_position;
        let map = self.base.map.borrow();
        use pathfinding::prelude::dijkstra;
        dijkstra(
            &start,
            |&p| {
                let mut successors = vec![];
                let tile = map.get_tile(IVec2::new(p.x as i32, p.y as i32));
                // Tunnel wrap: if currently in a tunnel, add the opposite exit as a neighbor
                if let Some(MapTile::Tunnel) = tile {
                    if p.x == 0 {
                        successors.push((UVec2::new(BOARD_CELL_SIZE.x - 2, p.y), 1));
                    } else if p.x == BOARD_CELL_SIZE.x - 1 {
                        successors.push((UVec2::new(1, p.y), 1));
                    }
                }
                for dir in &[Direction::Up, Direction::Down, Direction::Left, Direction::Right] {
                    let offset = dir.offset();
                    let next_p = IVec2::new(p.x as i32 + offset.x, p.y as i32 + offset.y);
                    if let Some(tile) = map.get_tile(next_p) {
                        if tile == MapTile::Wall {
                            continue;
                        }
                        let next_u = UVec2::new(next_p.x as u32, next_p.y as u32);
                        successors.push((next_u, 1));
                    }
                }
                successors
            },
            |&p| p == target,
        )
    }

    /// Changes the ghost's mode and handles direction reversal
    pub fn set_mode(&mut self, new_mode: GhostMode) {
        // Don't reverse if going to/from frightened or if in house
        let should_reverse = !matches!(self.mode, GhostMode::House(_))
            && !matches!(new_mode, GhostMode::House(_))
            && !matches!(self.mode, GhostMode::Frightened)
            && !matches!(new_mode, GhostMode::Frightened);

        self.mode = new_mode;

        self.base.speed.set_speed(match new_mode {
            GhostMode::Chase => 0.9375,
            GhostMode::Scatter => 0.85,
            GhostMode::Frightened => 0.7,
            GhostMode::Eyes => 1.5,
            GhostMode::House(_) => 0.7,
        });

        if should_reverse {
            self.base.set_direction_if_valid(self.base.direction.opposite());
        }
    }

    pub fn tick(&mut self) {
        if let GhostMode::House(house_mode) = self.mode {
            match house_mode {
                HouseMode::Waiting => {
                    // Ghosts in waiting mode move up and down
                    if self.base.is_grid_aligned() {
                        self.base.update_cell_position();

                        // Simple up and down movement
                        let current_pos = self.base.base.cell_position;
                        let start_pos = UVec2::new(13, 14); // Center of ghost house

                        if current_pos.y > start_pos.y + 1 {
                            // Too far down, move up
                            self.base.set_direction_if_valid(Direction::Up);
                        } else if current_pos.y < start_pos.y - 1 {
                            // Too far up, move down
                            self.base.set_direction_if_valid(Direction::Down);
                        } else if self.base.direction == Direction::Up {
                            // At top, switch to down
                            self.base.set_direction_if_valid(Direction::Down);
                        } else if self.base.direction == Direction::Down {
                            // At bottom, switch to up
                            self.base.set_direction_if_valid(Direction::Up);
                        }
                    }
                }
                HouseMode::Exiting => {
                    // Ghosts exiting move towards the exit
                    if self.base.is_grid_aligned() {
                        self.base.update_cell_position();

                        let exit_pos = UVec2::new(13, 11);
                        let current_pos = self.base.base.cell_position;

                        // Determine direction to exit
                        if current_pos.y > exit_pos.y {
                            // Need to move up
                            self.base.set_direction_if_valid(Direction::Up);
                        } else if current_pos.y == exit_pos.y && current_pos.x != exit_pos.x {
                            // At exit level, move horizontally to center
                            if current_pos.x < exit_pos.x {
                                self.base.set_direction_if_valid(Direction::Right);
                            } else {
                                self.base.set_direction_if_valid(Direction::Left);
                            }
                        } else if current_pos == exit_pos {
                            // Reached exit, transition to chase mode
                            self.mode = GhostMode::Chase;
                            self.current_house_offset = 0; // Reset offset
                        }
                    }
                }
                HouseMode::Entering => {
                    // Ghosts entering move towards their starting position
                    if self.base.is_grid_aligned() {
                        self.base.update_cell_position();

                        let start_pos = UVec2::new(13, 14); // Center of ghost house
                        let current_pos = self.base.base.cell_position;

                        // Determine direction to starting position
                        if current_pos.y < start_pos.y {
                            // Need to move down
                            self.base.set_direction_if_valid(Direction::Down);
                        } else if current_pos.y == start_pos.y && current_pos.x != start_pos.x {
                            // At house level, move horizontally to center
                            if current_pos.x < start_pos.x {
                                self.base.set_direction_if_valid(Direction::Right);
                            } else {
                                self.base.set_direction_if_valid(Direction::Left);
                            }
                        } else if current_pos == start_pos {
                            // Reached starting position, switch to waiting
                            self.mode = GhostMode::House(HouseMode::Waiting);
                        }
                    }
                }
            }

            // Update house offset for smooth transitions
            if self.current_house_offset != 0 {
                // Gradually reduce offset when turning
                if self.base.direction == Direction::Left || self.base.direction == Direction::Right {
                    if self.current_house_offset > 0 {
                        self.current_house_offset -= 1;
                    } else if self.current_house_offset < 0 {
                        self.current_house_offset += 1;
                    }
                }
            }

            self.base.tick();
            self.texture.tick();
            self.frightened_texture.tick();
            self.eyes_texture.tick();
            return;
        }

        // Normal ghost behavior
        if self.base.is_grid_aligned() {
            self.base.update_cell_position();
            if !self.base.handle_tunnel() {
                // Pathfinding logic (only if not in tunnel)
                if let Some(target_tile) = self.get_target_tile() {
                    if let Some((path, _)) = self.get_path_to_target(target_tile.as_uvec2()) {
                        if path.len() > 1 {
                            let next_move = path[1];
                            let x = self.base.base.cell_position.x;
                            let y = self.base.base.cell_position.y;
                            let dx = next_move.x as i32 - x as i32;
                            let dy = next_move.y as i32 - y as i32;
                            let new_direction = if dx > 0 {
                                Direction::Right
                            } else if dx < 0 {
                                Direction::Left
                            } else if dy > 0 {
                                Direction::Down
                            } else {
                                Direction::Up
                            };
                            self.base.set_direction_if_valid(new_direction);
                        }
                    }
                }
            }
        }

        // Handle house offset transition when turning
        if self.current_house_offset != 0 {
            if self.base.direction == Direction::Left || self.base.direction == Direction::Right {
                if self.current_house_offset > 0 {
                    self.current_house_offset -= 1;
                } else if self.current_house_offset < 0 {
                    self.current_house_offset += 1;
                }
            }
        }

        self.base.tick(); // Handles wall collision and movement
        self.texture.tick();
        self.frightened_texture.tick();
        self.eyes_texture.tick();
    }
}

impl Moving for Ghost {
    fn tick_movement(&mut self) {
        self.base.tick_movement();
    }
    fn tick(&mut self) {
        self.base.tick();
    }
    fn update_cell_position(&mut self) {
        self.base.update_cell_position();
    }
    fn next_cell(&self, direction: Option<Direction>) -> IVec2 {
        self.base.next_cell(direction)
    }
    fn is_wall_ahead(&self, direction: Option<Direction>) -> bool {
        self.base.is_wall_ahead(direction)
    }
    fn handle_tunnel(&mut self) -> bool {
        self.base.handle_tunnel()
    }
    fn is_grid_aligned(&self) -> bool {
        self.base.is_grid_aligned()
    }
    fn set_direction_if_valid(&mut self, new_direction: Direction) -> bool {
        self.base.set_direction_if_valid(new_direction)
    }
}

impl Renderable for Ghost {
    fn render(&mut self, canvas: &mut WindowCanvas) -> Result<()> {
        let mut pos = self.base.base.pixel_position;
        let dir = self.base.direction;

        // Apply house offset if in house mode or transitioning
        if matches!(self.mode, GhostMode::House(_)) || self.current_house_offset != 0 {
            pos.x += self.current_house_offset;
        }

        match self.mode {
            GhostMode::Frightened => {
                let tile = self.frightened_texture.animation.current_tile();
                let dest = sdl2::rect::Rect::new(pos.x - 4, pos.y - 4, tile.size.x as u32, tile.size.y as u32);
                self.frightened_texture.render(canvas, dest)
            }
            GhostMode::Eyes => {
                let tile = self.eyes_texture.up.first().unwrap();
                let dest = sdl2::rect::Rect::new(pos.x - 4, pos.y - 4, tile.size.x as u32, tile.size.y as u32);
                self.eyes_texture.render(canvas, dest, dir)
            }
            _ => {
                let tile = self.texture.up.first().unwrap();
                let dest = sdl2::rect::Rect::new(pos.x - 4, pos.y - 4, tile.size.x as u32, tile.size.y as u32);
                self.texture.render(canvas, dest, dir)
            }
        }
    }
}
