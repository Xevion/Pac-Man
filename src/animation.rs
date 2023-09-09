use sdl2::{
    rect::Rect,
    render::{Canvas, Texture},
    video::Window,
};

pub struct AnimatedTexture<'a> {
    raw_texture: Texture<'a>,
    frame_count: u32,
    frame_width: u32,
    frame_height: u32,
}

impl<'a> AnimatedTexture<'a> {
    pub fn new(
        texture: &'a Texture<'a>,
        frame_count: u32,
        frame_width: u32,
        frame_height: u32,
    ) -> Self {
        AnimatedTexture {
            raw_texture: texture,
            current_frame: 0,
            frame_count,
            frame_width,
            frame_height,
        }
    }

    fn next_frame(&mut self) {
        self.current_frame = (self.current_frame + 1) % self.frame_count;
    }

    fn get_frame_rect(&self) -> Rect {
        Rect::new(
            (self.current_frame * self.frame_width) as i32,
            0,
            self.frame_width,
            self.frame_height,
        )
    }

    pub fn render(&mut self, canvas: &mut Canvas<Window>, position: (i32, i32)) {
        let frame_rect = self.get_frame_rect();
        let position_rect = Rect::new(position.0, position.1, self.frame_width, self.frame_height);
        canvas
            .copy(&self.raw_texture, frame_rect, position_rect)
            .expect("Could not render sprite on canvas");

            self.next_frame();
    }
}
