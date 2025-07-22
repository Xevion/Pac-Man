use pathfinding::prelude::astar;
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
    constants::{MapTile, BOARD_OFFSET, CELL_SIZE},
    direction::Direction,
    entity::Entity,
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
    /// The absolute position of the ghost on the board, in pixels
    pub pixel_position: (i32, i32),
    /// The position of the ghost on the board, in grid coordinates
    pub cell_position: (u32, u32),
    /// The current direction of the ghost
    pub direction: Direction,
    /// The current mode of the ghost
    pub mode: GhostMode,
    /// The type/personality of this ghost
    pub ghost_type: GhostType,
    /// Whether the ghost is currently blue (frightened)
    pub is_blue: bool,
    /// Reference to the game map
    pub map: Rc<RefCell<Map>>,
    /// Reference to Pac-Man for targeting
    pub pacman: Rc<RefCell<Pacman<'a>>>,
    /// Movement speed
    speed: u32,
    /// Movement modulator
    modulation: SimpleTickModulator,
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

        Ghost {
            pixel_position: Map::cell_to_pixel(starting_position),
            cell_position: starting_position,
            direction: Direction::Left,
            mode: GhostMode::Chase,
            ghost_type,
            is_blue: false,
            map,
            pacman,
            speed: 3,
            modulation: SimpleTickModulator::new(1.0),
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
                .render(canvas, self.pixel_position, Direction::Right);
        }

        // Always render eyes on top
        let eye_frame = if self.mode == GhostMode::Frightened {
            4 // Frightened frame
        } else {
            match self.direction {
                Direction::Right => 0,
                Direction::Up => 1,
                Direction::Left => 2,
                Direction::Down => 3,
            }
        };

        self.eyes_sprite.render_static(
            canvas,
            self.pixel_position,
            Direction::Right,
            Some(eye_frame),
        );
    }

    /// Calculates the path to the target tile using the A* algorithm.
    fn get_path_to_target(&self, target: (u32, u32)) -> Option<(Vec<(u32, u32)>, u32)> {
        let start = self.cell_position;
        let map = self.map.borrow();

        astar(
            &start,
            |&p| {
                let mut successors = vec![];
                for dir in &[
                    Direction::Up,
                    Direction::Down,
                    Direction::Left,
                    Direction::Right,
                ] {
                    let (dx, dy) = dir.offset();
                    let next_p = (p.0 as i32 + dx, p.1 as i32 + dy);
                    if let Some(tile) = map.get_tile(next_p) {
                        if tile != MapTile::Wall {
                            successors.push(((next_p.0 as u32, next_p.1 as u32), 1));
                        }
                    }
                }
                successors
            },
            |&p| {
                ((p.0 as i32 - target.0 as i32).abs() + (p.1 as i32 - target.1 as i32).abs()) as u32
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
        let mut rng = rand::thread_rng();
        let (x, y) = self.cell_position;
        let mut possible_moves = Vec::new();

        // Check all four directions
        for dir in &[
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
        ] {
            // Don't allow reversing direction
            if *dir == self.direction.opposite() {
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
            let (dx, dy) = self.direction.opposite().offset();
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
        (pacman.cell_position.0 as i32, pacman.cell_position.1 as i32)
    }

    /// Changes the ghost's mode and handles direction reversal
    pub fn set_mode(&mut self, new_mode: GhostMode) {
        // Don't reverse if going to/from frightened or if in house
        let should_reverse = self.mode != GhostMode::House
            && new_mode != GhostMode::Frightened
            && self.mode != GhostMode::Frightened;

        self.mode = new_mode;

        if should_reverse {
            self.direction = self.direction.opposite();
        }
    }
}

impl Entity for Ghost<'_> {
    fn position(&self) -> (i32, i32) {
        self.pixel_position
    }

    fn cell_position(&self) -> (u32, u32) {
        self.cell_position
    }

    fn internal_position(&self) -> (u32, u32) {
        let (x, y) = self.position();
        (x as u32 % CELL_SIZE, y as u32 % CELL_SIZE)
    }

    fn is_colliding(&self, other: &dyn Entity) -> bool {
        let (x, y) = self.position();
        let (other_x, other_y) = other.position();
        x == other_x && y == other_y
    }

    fn tick(&mut self) {
        if self.mode == GhostMode::House {
            // For now, do nothing in the house
            return;
        }

        if self.internal_position() == (0, 0) {
            self.cell_position = (
                (self.pixel_position.0 as u32 / CELL_SIZE) - BOARD_OFFSET.0,
                (self.pixel_position.1 as u32 / CELL_SIZE) - BOARD_OFFSET.1,
            );

            // Pathfinding logic
            let target_tile = self.get_target_tile();
            if let Some((path, _)) =
                self.get_path_to_target((target_tile.0 as u32, target_tile.1 as u32))
            {
                if path.len() > 1 {
                    let next_move = path[1];
                    let (x, y) = self.cell_position;
                    let dx = next_move.0 as i32 - x as i32;
                    let dy = next_move.1 as i32 - y as i32;
                    self.direction = if dx > 0 {
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

            // Check if the next tile in the current direction is a wall
            let (dx, dy) = self.direction.offset();
            let next_cell = (
                self.cell_position.0 as i32 + dx,
                self.cell_position.1 as i32 + dy,
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

        if !self.modulation.next() {
            return;
        }

        // Update position based on current direction and speed
        let speed = self.speed as i32;
        match self.direction {
            Direction::Right => self.pixel_position.0 += speed,
            Direction::Left => self.pixel_position.0 -= speed,
            Direction::Up => self.pixel_position.1 -= speed,
            Direction::Down => self.pixel_position.1 += speed,
        }

        // Update cell position when aligned with grid
        if self.internal_position() == (0, 0) {
            self.cell_position = (
                (self.pixel_position.0 as u32 / CELL_SIZE) - BOARD_OFFSET.0,
                (self.pixel_position.1 as u32 / CELL_SIZE) - BOARD_OFFSET.1,
            );
        }
    }
}
