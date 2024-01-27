use crate::{Component, ComponentKind, Diagram};

const COMPONENT_HEIGHT: f64 = 50.0;
const COMPONENT_MARGIN_BOTTOM: f64 = 20.0;

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

pub trait Renderer {
    fn setup(self, width: f64, height: f64) -> Self;
    fn draw_line(self, from: &Vector2, to: &Vector2) -> Self;
    fn draw_circle(self, radius: f64, centre: &Vector2) -> Self;
    fn draw_text(self, text: &str, containment: &Rectangle) -> Self;
    fn draw_rectangle(self, rectangle: &Rectangle) -> Self;
    fn draw_text_with_rectangle(self, text: &str, rectangle: &Rectangle) -> Self;
    fn into_bytes(self) -> Vec<u8>;
}

struct Canvas {
    height: f64,
    width: f64,
    causes_container_height: f64,
    consequences_container_height: f64,
}

pub(crate) fn render_diagram<R>(r: R, diagram: &Diagram) -> Vec<u8>
where
    R: Renderer,
{
    let causes = diagram
        .components
        .iter()
        .filter(|c| c.kind == ComponentKind::Cause)
        .collect::<Vec<&Component>>();
    let consequences = diagram
        .components
        .iter()
        .filter(|c| c.kind == ComponentKind::Consequence)
        .collect::<Vec<&Component>>();
    let (r, canvas) = setup_canvas(r, &causes, &consequences);
    // Draw a border around the canvas, mostly for debugging purposes.
    let r = r.draw_rectangle(&Rectangle {
        centre: Vector2 {
            x: canvas.width / 2.0,
            y: canvas.height / 2.0,
        },
        width: canvas.width,
        height: canvas.height,
    });
    let r = render_components(r, &causes, ComponentKind::Cause, &canvas);
    let r = render_components(r, &consequences, ComponentKind::Consequence, &canvas);
    let r = render_event_circle(r, &diagram, &canvas);
    r.into_bytes()
}

fn render_event_circle<R>(r: R, diagram: &Diagram, canvas: &Canvas) -> R
where
    R: Renderer,
{
    let width = (diagram.event.len() as f64) * 13.0 + 5.0;
    let radius = width / 2.0;
    let r = r.draw_circle(
        radius,
        &Vector2 {
            x: canvas.width / 2.0,
            y: canvas.height / 2.0,
        },
    );
    let r = r.draw_text(
        &diagram.event,
        &Rectangle {
            centre: Vector2 {
                x: canvas.width / 2.0,
                y: canvas.height / 2.0,
            },
            width: radius,
            height: radius,
        },
    );
    r
}

fn setup_canvas<'a, R>(r: R, causes: &[&Component], consequences: &[&Component]) -> (R, Canvas)
where
    R: Renderer,
{
    let causes_container_height = calculate_components_container_height(causes);
    let consequences_container_height = calculate_components_container_height(consequences);
    let max_container_height = causes_container_height.max(consequences_container_height);
    let canvas_height = max_container_height * 1.1 + 50.0;
    let canvas = Canvas {
        height: canvas_height,
        width: 2000.0,
        causes_container_height,
        consequences_container_height,
    };
    let r = r.setup(canvas.width, canvas.height);
    (r, canvas)
}

fn calculate_components_container_height<'a>(components: &[&Component]) -> f64 {
    let components_count = components.len() as f64;
    components_count * COMPONENT_HEIGHT + ((components_count - 1.0) * COMPONENT_MARGIN_BOTTOM)
}

fn render_components<'a, R>(
    mut r: R,
    components: &[&Component],
    kind: ComponentKind,
    canvas: &Canvas,
) -> R
where
    R: Renderer,
{
    let is_cause = kind == ComponentKind::Cause;
    let container_height = if is_cause {
        canvas.causes_container_height
    } else {
        canvas.consequences_container_height
    };
    let components_container_top = (canvas.height / 2.0) - (container_height / 2.0);
    let longest_name = components.iter().map(|c| c.name.len()).max().unwrap_or(0) as f64;
    for (i, component) in components.iter().enumerate().map(|(i, c)| (i as f64, c)) {
        let height = 50.0;
        let y_relative = i * COMPONENT_HEIGHT + (i * COMPONENT_MARGIN_BOTTOM);
        let y = components_container_top + y_relative + (height / 2.0);
        let box_width = longest_name * 15.0;
        let x = if is_cause {
            (box_width / 2.0) + 10.0
        } else {
            canvas.width - (box_width / 2.0) - 10.0
        };
        let rectangle = Rectangle {
            centre: Vector2 { x, y },
            width: box_width,
            height,
        };
        r = r.draw_text_with_rectangle(&component.name, &rectangle);
    }
    r
}

impl Rectangle {
    pub fn with_padding(mut self, padding: f64) -> Self {
        self.width += padding;
        self.height += padding;
        self
    }
}
