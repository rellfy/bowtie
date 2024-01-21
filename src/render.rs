use crate::Diagram;

#[derive(Clone, Copy)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Copy)]
pub struct Rectangle {
    pub centre: Point,
    pub width: f64,
    pub height: f64,
}

pub trait Renderer {
    fn setup(self, width: f64, height: f64) -> Self;
    fn draw_line(self, from: &Point, to: &Point) -> Self;
    fn draw_circle(self, radius: f64) -> Self;
    fn draw_text(self, text: &str, containment: &Rectangle) -> Self;
    fn draw_rectangle(self, rectangle: &Rectangle) -> Self;
    fn draw_text_with_rectangle(self, text: &str, centre: &Point) -> Self;
    fn into_bytes(self) -> Vec<u8>;
}

pub fn render_diagram<R>(r: R, diagram: &Diagram) -> Vec<u8>
where
    R: Renderer,
{
    let r = r.setup(1000.0, 500.0);
    let r = r.draw_text_with_rectangle("some_text", &Point { x: 100.0, y: 80.0 });
    r.into_bytes()
}

impl Rectangle {
    pub fn with_padding(mut self, padding: f64) -> Self {
        self.width += padding;
        self.height += padding;
        self
    }
}
