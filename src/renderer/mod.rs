mod svg;
pub use svg::SvgRenderer;

#[derive(Clone, Copy, Debug)]
pub struct Vector2 {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Copy)]
pub struct Rectangle {
    pub centre: Vector2,
    pub width: f64,
    pub height: f64,
}

#[derive(Copy, Clone)]
pub enum Alignment {
    Center,
    Left,
    Right,
}

pub trait Renderer {
    fn setup(self, width: f64, height: f64) -> Self;
    fn draw_line(self, from: &Vector2, to: &Vector2) -> Self;
    fn draw_circle(self, radius: f64, centre: &Vector2) -> Self;
    fn draw_text(self, text: &str, containment: &Rectangle, alignment: Alignment) -> Self;
    fn draw_rectangle(self, rectangle: &Rectangle) -> Self;
    fn draw_text_with_rectangle(
        self,
        text: &str,
        rectangle: &Rectangle,
        alignment: Alignment,
    ) -> Self;
    fn into_bytes(self) -> Vec<u8>;
}

impl Rectangle {
    pub fn with_padding(mut self, padding: f64) -> Self {
        self.width += padding;
        self.height += padding;
        self
    }
}
