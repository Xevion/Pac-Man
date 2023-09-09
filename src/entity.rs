pub trait Entity {
    // Returns true if the entity is colliding with the other entity
    fn is_colliding(&self, other: &dyn Entity) -> bool;
    // Returns the absolute position of the entity
    fn position(&self) -> (i32, i32);
    // Returns the cell position of the entity (XY position within the grid)
    fn cell_position(&self) -> (u32, u32);
    // Tick the entity (move it, perform collision checks, etc) 
    fn tick(&mut self);
}