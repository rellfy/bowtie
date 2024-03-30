use crate::renderer::{Rectangle, Renderer, Vector2};
use svg::node::element::path::Data;
use svg::node::element::{Circle, Path, Text};
use svg::Document;

const FONT_WIDTH: f64 = 1.8;
const FONT_FAMILY: &str = "Courier, monospace";
const DEFAULT_BG_FILL: &str = "white";

pub struct SvgRenderer {
    document: Document,
    stroke_width: u32,
}

impl SvgRenderer {
    pub fn new() -> Self {
        SvgRenderer {
            document: Document::new(),
            stroke_width: 3,
        }
    }
}

impl Renderer for SvgRenderer {
    fn setup(mut self, width: f64, height: f64) -> Self {
        self.document = Document::new().set("viewBox", (0, 0, width, height));
        self
    }

    fn draw_line(mut self, from: &Vector2, to: &Vector2) -> Self {
        let data = Data::new().move_to((from.x, from.y)).line_to((to.x, to.y));
        let path = Path::new()
            .set("fill", "none")
            .set("stroke", "black")
            .set("stroke-width", self.stroke_width)
            .set("d", data);
        self.document = self.document.add(path);
        self
    }

    fn draw_circle(mut self, radius: f64, centre: &Vector2) -> Self {
        let circle = Circle::new()
            .set("cx", centre.x)
            .set("cy", centre.y)
            .set("r", radius)
            .set("stroke", "black")
            .set("stroke-width", self.stroke_width)
            .set("fill", DEFAULT_BG_FILL);
        self.document = self.document.add(circle);
        self
    }

    fn draw_text(mut self, text: &str, containment: &Rectangle) -> Self {
        let font_size = 18.0;
        let width = (text.len() as f64) * font_size / FONT_WIDTH;
        let y = containment.centre.y + (font_size / (FONT_WIDTH * 2.0));
        let x = containment.centre.x - (width / FONT_WIDTH);
        let text = Text::new()
            .set("x", x)
            .set("y", y)
            .set("font-size", font_size)
            .set("fill", "black")
            .set("font-family", FONT_FAMILY)
            .add(svg::node::Text::new(text));
        self.document = self.document.add(text);
        self
    }

    fn draw_rectangle(mut self, rectangle: &Rectangle) -> Self {
        let top_left = Vector2 {
            x: rectangle.centre.x - (rectangle.width / 2.0),
            y: rectangle.centre.y - (rectangle.height / 2.0),
        };
        let data = Data::new()
            .move_to((top_left.x, top_left.y))
            .line_by((rectangle.width, 0))
            .line_by((0, rectangle.height))
            .line_by((-rectangle.width, 0))
            .close();
        let path = Path::new()
            .set("fill", DEFAULT_BG_FILL)
            .set("stroke", "black")
            .set("stroke-width", self.stroke_width)
            .set("font-family", FONT_FAMILY)
            .set("d", data);
        self.document = self.document.add(path);
        self
    }

    fn draw_text_with_rectangle(mut self, text: &str, rectangle: &Rectangle) -> Self {
        self = self.draw_rectangle(&rectangle.with_padding(2.0));
        self = self.draw_text(text, &rectangle);
        self
    }

    fn into_bytes(self) -> Vec<u8> {
        let mut bytes = Vec::<u8>::new();
        svg::write(&mut bytes, &self.document).unwrap();
        bytes
    }
}
