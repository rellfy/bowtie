//! This module is responsible for drawing on the canvas relying on
//! a renderer and its context.
use crate::renderer::{Rectangle, Renderer, Vector2};
use crate::{Component, ComponentKind, Diagram};
use std::collections::HashSet;

const COMPONENT_HEIGHT: f64 = 50.0;
const BARRIER_WIDTH: f64 = 50.0;
const COMPONENT_MARGIN_BOTTOM: f64 = 20.0;
const COMPONENT_PADDING_X: f64 = 10.0;
const BARRIER_MARGIN_RIGHT: f64 = 50.0;
const BARRIERS_CONTAINER_HORIZONTAL_PADDING: f64 = 150.0;

struct Brush<R: Renderer> {
    renderer: R,
    context: Context,
    diagram: Diagram,
}

/// Holds state variables for rendering purposes.
struct Context {}

struct Canvas {
    height: f64,
    width: f64,
    causes_container_height: f64,
    consequences_container_height: f64,
    max_component_box_width: f64,
    circle_left_point: Option<Vector2>,
    circle_right_point: Option<Vector2>,
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
    let (r, mut canvas) = setup_canvas(
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
    let r = render_components(r, &causes, ComponentKind::Cause, &canvas);
    let r = render_components(r, &consequences, ComponentKind::Consequence, &canvas);
    let r = render_event_circle(r, &diagram, &mut canvas);
    let r = render_barrier_lines(r, &canvas, &causes, ComponentKind::Cause);
    let r = render_barrier_lines(r, &canvas, &consequences, ComponentKind::Consequence);
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

fn render_event_circle<R>(r: R, diagram: &Diagram, canvas: &mut Canvas) -> R
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
    canvas.circle_left_point = Some(Vector2 {
        x: canvas.width / 2.0 - radius,
        y: canvas.height / 2.0,
    });
    canvas.circle_right_point = Some(Vector2 {
        x: canvas.width / 2.0 + radius,
        y: canvas.height / 2.0,
    });
    r
}

fn render_barrier_lines<R>(
    mut r: R,
    canvas: &Canvas,
    components: &[&Component],
    kind: ComponentKind,
) -> R
where
    R: Renderer,
{
    let circle_point = if kind == ComponentKind::Cause {
        canvas.circle_left_point.as_ref().unwrap()
    } else {
        canvas.circle_right_point.as_ref().unwrap()
    };
    for (i, _) in components.into_iter().enumerate() {
        let y = get_component_y_center(i as f64, &kind, &canvas);
        let x_center = get_component_x_center(&kind, &canvas);
        let x_edge = match kind {
            ComponentKind::Cause => x_center + canvas.max_component_box_width / 2.0,
            ComponentKind::Consequence => x_center - canvas.max_component_box_width / 2.0,
        };
        let component_point = Vector2 { x: x_edge, y };
        r = r.draw_line(&component_point, &circle_point);
    }
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
        max_component_box_width,
        circle_left_point: None,
        circle_right_point: None,
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
) -> R
where
    R: Renderer,
{
    for (i, component) in components.iter().enumerate().map(|(i, c)| (i as f64, c)) {
        let y = get_component_y_center(i, &kind, &canvas);
        let x = get_component_x_center(&kind, &canvas);
        let rectangle = Rectangle {
            centre: Vector2 { x, y },
            width: canvas.max_component_box_width,
            height: COMPONENT_HEIGHT,
        };
        r = r.draw_text_with_rectangle(&component.name, &rectangle);
    }
    r
}

fn get_component_x_center(kind: &ComponentKind, canvas: &Canvas) -> f64 {
    match kind {
        ComponentKind::Cause => (canvas.max_component_box_width / 2.0) + COMPONENT_PADDING_X,
        ComponentKind::Consequence => {
            canvas.width - (canvas.max_component_box_width / 2.0) - COMPONENT_PADDING_X
        }
    }
}

fn get_component_y_center(i: f64, kind: &ComponentKind, canvas: &Canvas) -> f64 {
    let container_height = match kind {
        ComponentKind::Cause => canvas.causes_container_height,
        ComponentKind::Consequence => canvas.consequences_container_height,
    };
    let components_container_top = (canvas.height / 2.0) - (container_height / 2.0);
    let y_relative = i * COMPONENT_HEIGHT + (i * COMPONENT_MARGIN_BOTTOM);
    components_container_top + y_relative + (COMPONENT_HEIGHT / 2.0)
}

pub fn text_width(text: &str) -> f64 {
    text.len() as f64 * 15.0
}
