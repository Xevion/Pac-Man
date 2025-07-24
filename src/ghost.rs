use rand::Rng;

use crate::animation::{AnimatedAtlasTexture, FrameDrawn};
use crate::constants::{MapTile, BOARD_WIDTH};
use crate::direction::Direction;
use crate::entity::{Entity, MovableEntity, Moving, Renderable};
use crate::map::Map;
use crate::modulation::{SimpleTickModulator, TickModulator};
use crate::pacman::Pacman;
use glam::{IVec2, UVec2};
use sdl2::pixels::Color;
use sdl2::render::Texture;
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
    House,
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
pub struct Ghost<'a> {
    /// Shared movement and position fields.
    pub base: MovableEntity,
    /// The current mode of the ghost
    pub mode: GhostMode,
    /// The type/personality of this ghost
    pub ghost_type: GhostType,
    /// Reference to Pac-Man for targeting
    pub pacman: Rc<RefCell<Pacman<'a>>>,
    pub body_sprite: AnimatedAtlasTexture<'a>,
    pub eyes_sprite: AnimatedAtlasTexture<'a>,
}

impl Ghost<'_> {
    /// Creates a new ghost instance
    pub fn new<'a>(
        ghost_type: GhostType,
        starting_position: UVec2,
        body_texture: Texture<'a>,
        eyes_texture: Texture<'a>,
        map: Rc<RefCell<Map>>,
        pacman: Rc<RefCell<Pacman<'a>>>,
    ) -> Ghost<'a> {
        let color = ghost_type.color();
        let mut body_sprite = AnimatedAtlasTexture::new(body_texture, 8, 2, 32, 32, Some((-4, -4)));
        body_sprite.set_color_modulation(color.r, color.g, color.b);
        let pixel_position = Map::cell_to_pixel(starting_position);
        Ghost {
            base: MovableEntity::new(
                pixel_position,
                starting_position,
                Direction::Left,
                3,
                SimpleTickModulator::new(1.0),
                map,
            ),
            mode: GhostMode::Chase,
            ghost_type,
            pacman,
            body_sprite,
            eyes_sprite: AnimatedAtlasTexture::new(eyes_texture, 1, 4, 32, 32, Some((-4, -4))),
        }
    }

    /// Gets the target tile for this ghost based on its current mode
    pub fn get_target_tile(&self) -> IVec2 {
        match self.mode {
            GhostMode::Scatter => self.get_scatter_target(),
            GhostMode::Chase => self.get_chase_target(),
            GhostMode::Frightened => self.get_random_target(),
            GhostMode::Eyes => self.get_house_target(),
            GhostMode::House => self.get_house_exit_target(),
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
        let mut rng = rand::rng();
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

    /// Gets the exit point target when leaving house
    fn get_house_exit_target(&self) -> IVec2 {
        IVec2::new(13, 11) // Just above ghost house
    }

    /// Gets this ghost's chase mode target (to be implemented by each ghost type)
    fn get_chase_target(&self) -> IVec2 {
        let pacman = self.pacman.borrow();
        let cell = pacman.base().cell_position;
        IVec2::new(cell.x as i32, cell.y as i32)
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
                        successors.push((UVec2::new(BOARD_WIDTH - 2, p.y), 1));
                    } else if p.x == BOARD_WIDTH - 1 {
                        successors.push((UVec2::new(1, p.y), 1));
                    }
                }
                for dir in &[Direction::Up, Direction::Down, Direction::Left, Direction::Right] {
                    let (dx, dy) = dir.offset();
                    let next_p = IVec2::new(p.x as i32 + dx, p.y as i32 + dy);
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
        let should_reverse =
            self.mode != GhostMode::House && new_mode != GhostMode::Frightened && self.mode != GhostMode::Frightened;

        self.mode = new_mode;

        self.base.speed = match new_mode {
            GhostMode::Chase => 3,
            GhostMode::Scatter => 2,
            GhostMode::Frightened => 2,
            GhostMode::Eyes => 7,
            GhostMode::House => 0,
        };

        if should_reverse {
            self.base.set_direction_if_valid(self.base.direction.opposite());
        }
    }

    pub fn tick(&mut self) {
        if self.mode == GhostMode::House {
            // For now, do nothing in the house
            return;
        }

        if self.base.is_grid_aligned() {
            self.base.update_cell_position();

            if !self.base.handle_tunnel() {
                // Pathfinding logic (only if not in tunnel)
                let target_tile = self.get_target_tile();
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

            // Don't move if the next tile is a wall
            if self.base.is_wall_ahead(None) {
                return;
            }
        }

        if self.base.modulation.next() {
            self.base.move_forward();

            if self.base.is_grid_aligned() {
                self.base.update_cell_position();
            }
        }
    }
}

impl<'a> Moving for Ghost<'a> {
    fn move_forward(&mut self) {
        self.base.move_forward();
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

impl<'a> Renderable for Ghost<'a> {
    fn render(&self, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>) {
        let pos = self.base.base.pixel_position;
        self.body_sprite.render(canvas, (pos.x, pos.y), Direction::Right, None);
        // Inline the eye_frame logic here
        let eye_frame = if self.mode == GhostMode::Frightened {
            4 // Frightened frame
        } else {
            match self.base.direction {
                Direction::Right => 0,
                Direction::Up => 1,
                Direction::Left => 2,
                Direction::Down => 3,
            }
        };
        self.eyes_sprite
            .render(canvas, (pos.x, pos.y), Direction::Right, Some(eye_frame));
    }
}
