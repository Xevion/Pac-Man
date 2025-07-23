use pathfinding::prelude::dijkstra;
use sdl2::{
    pixels::Color,
    render::{Canvas, Texture},
    video::Window,
};
use std::cell::RefCell;
use std::rc::Rc;

use rand::Rng;

use crate::{
    animation::AnimatedTexture,
    constants::{MapTile, BOARD_OFFSET, BOARD_WIDTH, CELL_SIZE},
    direction::Direction,
    entity::{Entity, MovableEntity},
    map::Map,
    modulation::{SimpleTickModulator, TickModulator},
    pacman::Pacman,
};

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
    /// Reference to the game map
    pub map: Rc<RefCell<Map>>,
    /// Reference to Pac-Man for targeting
    pub pacman: Rc<RefCell<Pacman<'a>>>,
    /// Ghost body sprite
    body_sprite: AnimatedTexture<'a>,
    /// Ghost eyes sprite
    eyes_sprite: AnimatedTexture<'a>,
}

impl Ghost<'_> {
    /// Creates a new ghost instance
    pub fn new<'a>(
        ghost_type: GhostType,
        starting_position: (u32, u32),
        body_texture: Texture<'a>,
        eyes_texture: Texture<'a>,
        map: Rc<RefCell<Map>>,
        pacman: Rc<RefCell<Pacman<'a>>>,
    ) -> Ghost<'a> {
        let color = ghost_type.color();
        let mut body_sprite = AnimatedTexture::new(body_texture, 8, 2, 32, 32, Some((-4, -4)));
        body_sprite.set_color_modulation(color.r, color.g, color.b);
        let pixel_position = Map::cell_to_pixel(starting_position);
        Ghost {
            base: MovableEntity::new(
                pixel_position,
                starting_position,
                Direction::Left,
                3,
                SimpleTickModulator::new(1.0),
            ),
            mode: GhostMode::Chase,
            ghost_type,
            map,
            pacman,
            body_sprite,
            eyes_sprite: AnimatedTexture::new(eyes_texture, 1, 4, 32, 32, Some((-4, -4))),
        }
    }

    /// Renders the ghost to the canvas
    pub fn render(&mut self, canvas: &mut Canvas<Window>) {
        // Render body
        if self.mode != GhostMode::Eyes {
            let color = if self.mode == GhostMode::Frightened {
                Color::RGB(0, 0, 255)
            } else {
                self.ghost_type.color()
            };

            self.body_sprite
                .set_color_modulation(color.r, color.g, color.b);
            self.body_sprite
                .render(canvas, self.base.pixel_position, Direction::Right);
        }

        // Always render eyes on top
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

        self.eyes_sprite.render_static(
            canvas,
            self.base.pixel_position,
            Direction::Right,
            Some(eye_frame),
        );
    }

    /// Calculates the path to the target tile using the A* algorithm.
    pub fn get_path_to_target(&self, target: (u32, u32)) -> Option<(Vec<(u32, u32)>, u32)> {
        let start = self.base.cell_position;
        let map = self.map.borrow();

        dijkstra(
            &start,
            |&p| {
                let mut successors = vec![];
                let tile = map.get_tile((p.0 as i32, p.1 as i32));
                // Tunnel wrap: if currently in a tunnel, add the opposite exit as a neighbor
                if let Some(MapTile::Tunnel) = tile {
                    if p.0 == 0 {
                        successors.push(((BOARD_WIDTH - 2, p.1), 1));
                    } else if p.0 == BOARD_WIDTH - 1 {
                        successors.push(((1, p.1), 1));
                    }
                }
                for dir in &[
                    Direction::Up,
                    Direction::Down,
                    Direction::Left,
                    Direction::Right,
                ] {
                    let (dx, dy) = dir.offset();
                    let next_p = (p.0 as i32 + dx, p.1 as i32 + dy);
                    if let Some(tile) = map.get_tile(next_p) {
                        if tile == MapTile::Wall {
                            continue;
                        }
                        successors.push(((next_p.0 as u32, next_p.1 as u32), 1));
                    }
                }
                successors
            },
            |&p| p == target,
        )
    }

    /// Gets the target tile for this ghost based on its current mode
    pub fn get_target_tile(&self) -> (i32, i32) {
        match self.mode {
            GhostMode::Scatter => self.get_scatter_target(),
            GhostMode::Chase => self.get_chase_target(),
            GhostMode::Frightened => self.get_random_target(),
            GhostMode::Eyes => self.get_house_target(),
            GhostMode::House => self.get_house_exit_target(),
        }
    }

    /// Gets this ghost's home corner target for scatter mode
    fn get_scatter_target(&self) -> (i32, i32) {
        match self.ghost_type {
            GhostType::Blinky => (25, 0), // Top right
            GhostType::Pinky => (2, 0),   // Top left
            GhostType::Inky => (27, 35),  // Bottom right
            GhostType::Clyde => (0, 35),  // Bottom left
        }
    }

    /// Gets a random adjacent tile for frightened mode
    fn get_random_target(&self) -> (i32, i32) {
        let mut rng = rand::rng();
        let (x, y) = self.base.cell_position;
        let mut possible_moves = Vec::new();

        // Check all four directions
        for dir in &[
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
        ] {
            // Don't allow reversing direction
            if *dir == self.base.direction.opposite() {
                continue;
            }

            let (dx, dy) = dir.offset();
            let next_cell = (x as i32 + dx, y as i32 + dy);
            let tile = self.map.borrow().get_tile(next_cell);
            if let Some(MapTile::Wall) = tile {
                // It's a wall, not a valid move
            } else {
                possible_moves.push(next_cell);
            }
        }

        if possible_moves.is_empty() {
            // No valid moves, must reverse
            let (dx, dy) = self.base.direction.opposite().offset();
            return (x as i32 + dx, y as i32 + dy);
        }

        // Choose a random valid move
        possible_moves[rng.gen_range(0..possible_moves.len())]
    }

    /// Gets the ghost house target for returning eyes
    fn get_house_target(&self) -> (i32, i32) {
        (13, 14) // Center of ghost house
    }

    /// Gets the exit point target when leaving house
    fn get_house_exit_target(&self) -> (i32, i32) {
        (13, 11) // Just above ghost house
    }

    /// Gets this ghost's chase mode target (to be implemented by each ghost type)
    fn get_chase_target(&self) -> (i32, i32) {
        // Default implementation just targets Pac-Man directly
        let pacman = self.pacman.borrow();
        let cell = pacman.base.cell_position;
        (cell.0 as i32, cell.1 as i32)
    }

    /// Changes the ghost's mode and handles direction reversal
    pub fn set_mode(&mut self, new_mode: GhostMode) {
        // Don't reverse if going to/from frightened or if in house
        let should_reverse = self.mode != GhostMode::House
            && new_mode != GhostMode::Frightened
            && self.mode != GhostMode::Frightened;

        self.mode = new_mode;

        self.base.speed = match new_mode {
            GhostMode::Chase => 3,
            GhostMode::Scatter => 2,
            GhostMode::Frightened => 2,
            GhostMode::Eyes => 7,
            GhostMode::House => 0,
        };

        if should_reverse {
            self.base.direction = self.base.direction.opposite();
        }
    }
}

impl Entity for Ghost<'_> {
    fn base(&self) -> &MovableEntity {
        &self.base
    }

    /// Returns true if the ghost entity is colliding with the other entity.
    fn is_colliding(&self, other: &dyn Entity) -> bool {
        let (x, y) = self.base.pixel_position;
        let (other_x, other_y) = other.base().pixel_position;
        x == other_x && y == other_y
    }

    /// Ticks the ghost entity.
    fn tick(&mut self) {
        if self.mode == GhostMode::House {
            // For now, do nothing in the house
            return;
        }

        if self.base.internal_position() == (0, 0) {
            self.base.cell_position = (
                (self.base.pixel_position.0 as u32 / CELL_SIZE) - BOARD_OFFSET.0,
                (self.base.pixel_position.1 as u32 / CELL_SIZE) - BOARD_OFFSET.1,
            );

            let current_tile = self
                .map
                .borrow()
                .get_tile((
                    self.base.cell_position.0 as i32,
                    self.base.cell_position.1 as i32,
                ))
                .unwrap_or(MapTile::Empty);
            if current_tile == MapTile::Tunnel {
                self.base.in_tunnel = true;
            }

            // Tunnel logic: if in tunnel, force movement and prevent direction change
            if self.base.in_tunnel {
                // If out of bounds, teleport to the opposite side and exit tunnel
                if self.base.cell_position.0 == 0 {
                    self.base.cell_position.0 = BOARD_WIDTH - 2;
                    self.base.pixel_position =
                        Map::cell_to_pixel((self.base.cell_position.0, self.base.cell_position.1));
                    self.base.in_tunnel = false;
                } else if self.base.cell_position.0 == BOARD_WIDTH - 1 {
                    self.base.cell_position.0 = 1;
                    self.base.pixel_position =
                        Map::cell_to_pixel((self.base.cell_position.0, self.base.cell_position.1));
                    self.base.in_tunnel = false;
                } else {
                    // While in tunnel, do not allow direction change
                    // and always move in the current direction
                }
            } else {
                // Pathfinding logic (only if not in tunnel)
                let target_tile = self.get_target_tile();
                if let Some((path, _)) =
                    self.get_path_to_target((target_tile.0 as u32, target_tile.1 as u32))
                {
                    if path.len() > 1 {
                        let next_move = path[1];
                        let (x, y) = self.base.cell_position;
                        let dx = next_move.0 as i32 - x as i32;
                        let dy = next_move.1 as i32 - y as i32;
                        self.base.direction = if dx > 0 {
                            Direction::Right
                        } else if dx < 0 {
                            Direction::Left
                        } else if dy > 0 {
                            Direction::Down
                        } else {
                            Direction::Up
                        };
                    }
                }
            }

            // Check if the next tile in the current direction is a wall
            let (dx, dy) = self.base.direction.offset();
            let next_cell = (
                self.base.cell_position.0 as i32 + dx,
                self.base.cell_position.1 as i32 + dy,
            );
            let next_tile = self
                .map
                .borrow()
                .get_tile(next_cell)
                .unwrap_or(MapTile::Empty);
            if next_tile == MapTile::Wall {
                // Don't move if the next tile is a wall
                return;
            }
        }

        if !self.base.modulation.next() {
            return;
        }

        // Update position based on current direction and speed
        self.base.move_forward();

        // Update cell position when aligned with grid
        if self.base.internal_position() == (0, 0) {
            self.base.cell_position = (
                (self.base.pixel_position.0 as u32 / CELL_SIZE) - BOARD_OFFSET.0,
                (self.base.pixel_position.1 as u32 / CELL_SIZE) - BOARD_OFFSET.1,
            );
        }
    }
}
