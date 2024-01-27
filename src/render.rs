use crate::{Component, ComponentKind, Diagram};
use std::collections::HashSet;

const COMPONENT_HEIGHT: f64 = 50.0;
const BARRIER_WIDTH: f64 = 50.0;
const COMPONENT_MARGIN_BOTTOM: f64 = 20.0;
const BARRIER_MARGIN_RIGHT: f64 = 50.0;
const BARRIERS_CONTAINER_HORIZONTAL_PADDING: f64 = 150.0;

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
    let causes = filter_components(&diagram, ComponentKind::Cause);
    let consequences = filter_components(&diagram, ComponentKind::Consequence);
    let barriers_causes = filter_barriers(&causes);
    let barriers_consequences = filter_barriers(&consequences);
    let max_component_box_width = calculate_max_components_box_width(&causes, &consequences);
    let max_barrier_container_width =
        calculate_max_barriers_container_width(&barriers_causes, &barriers_consequences);
    println!(
        "max barrier container width: {}",
        max_barrier_container_width
    );
    let (r, canvas) = setup_canvas(
        r,
        &causes,
        &consequences,
        diagram,
        max_component_box_width,
        max_barrier_container_width,
    );
    // Draw a border around the canvas, mostly for debugging purposes.
    let r = r.draw_rectangle(&Rectangle {
        centre: Vector2 {
            x: canvas.width / 2.0,
            y: canvas.height / 2.0,
        },
        width: canvas.width,
        height: canvas.height,
    });
    let r = render_components(
        r,
        &causes,
        ComponentKind::Cause,
        &canvas,
        max_component_box_width,
    );
    let r = render_components(
        r,
        &consequences,
        ComponentKind::Consequence,
        &canvas,
        max_component_box_width,
    );
    let r = render_event_circle(r, &diagram, &canvas);
    r.into_bytes()
}

fn filter_components(diagram: &Diagram, kind: ComponentKind) -> Vec<&Component> {
    diagram
        .components
        .iter()
        .filter(|c| c.kind == kind)
        .collect::<Vec<&Component>>()
}

fn filter_barriers<'a>(components: &'a [&Component]) -> HashSet<&'a str> {
    let mut barriers = HashSet::<&str>::new();
    for component in components {
        for component_barrier in &component.barriers {
            barriers.insert(component_barrier);
        }
    }
    barriers
}

fn render_event_circle<R>(r: R, diagram: &Diagram, canvas: &Canvas) -> R
where
    R: Renderer,
{
    let radius = calculate_event_circle_radius(&diagram.event);
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

fn calculate_event_circle_radius(event: &str) -> f64 {
    let width = text_width(&event);
    width / 2.0
}

fn setup_canvas<'a, R>(
    r: R,
    causes: &[&Component],
    consequences: &[&Component],
    diagram: &Diagram,
    max_component_box_width: f64,
    max_barriers_container_width: f64,
) -> (R, Canvas)
where
    R: Renderer,
{
    let causes_container_height = calculate_components_container_height(causes);
    let consequences_container_height = calculate_components_container_height(consequences);
    let max_container_height = causes_container_height.max(consequences_container_height);
    let canvas_height = max_container_height * 1.1 + 50.0;
    let width = calculate_canvas_width(
        diagram,
        max_component_box_width,
        max_barriers_container_width,
    );
    let canvas = Canvas {
        height: canvas_height,
        width,
        causes_container_height,
        consequences_container_height,
    };
    let r = r.setup(canvas.width, canvas.height);
    (r, canvas)
}

fn calculate_components_container_height(components: &[&Component]) -> f64 {
    let components_count = components.len() as f64;
    components_count * COMPONENT_HEIGHT + ((components_count - 1.0) * COMPONENT_MARGIN_BOTTOM)
}

fn calculate_barriers_container_width(barriers: &HashSet<&str>) -> f64 {
    let barriers_count = barriers.len() as f64;
    let padding = BARRIERS_CONTAINER_HORIZONTAL_PADDING * 2.0;
    barriers_count * BARRIER_WIDTH + ((barriers_count - 1.0) * BARRIER_MARGIN_RIGHT) + padding
}

fn calculate_canvas_width(
    diagram: &Diagram,
    max_component_box_width: f64,
    max_barriers_container_width: f64,
) -> f64 {
    calculate_event_circle_radius(&diagram.event)
        + (max_component_box_width * 2.0)
        + (max_barriers_container_width * 2.0)
}

fn calculate_max_barriers_container_width(a: &HashSet<&str>, b: &HashSet<&str>) -> f64 {
    let aw = calculate_barriers_container_width(a);
    let bw = calculate_barriers_container_width(b);
    aw.max(bw)
}

fn calculate_max_components_box_width(a: &[&Component], b: &[&Component]) -> f64 {
    let aw = calculate_max_component_box_width(a);
    let bw = calculate_max_component_box_width(b);
    aw.max(bw)
}

fn calculate_max_component_box_width(components: &[&Component]) -> f64 {
    components
        .iter()
        .map(|c| text_width(&c.name) as u32)
        .max()
        .map(|v| v as f64)
        .unwrap_or(0.0)
}

fn render_components<'a, R>(
    mut r: R,
    components: &[&Component],
    kind: ComponentKind,
    canvas: &Canvas,
    max_component_box_width: f64,
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
    for (i, component) in components.iter().enumerate().map(|(i, c)| (i as f64, c)) {
        let height = 50.0;
        let y_relative = i * COMPONENT_HEIGHT + (i * COMPONENT_MARGIN_BOTTOM);
        let y = components_container_top + y_relative + (height / 2.0);
        let x = if is_cause {
            (max_component_box_width / 2.0) + 10.0
        } else {
            canvas.width - (max_component_box_width / 2.0) - 10.0
        };
        let rectangle = Rectangle {
            centre: Vector2 { x, y },
            width: max_component_box_width,
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

pub fn text_width(text: &str) -> f64 {
    text.len() as f64 * 15.0
}
