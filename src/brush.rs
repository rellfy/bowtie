//! This module is responsible for drawing on the canvas relying on
//! a renderer and its context.
use crate::renderer::{Alignment, Rectangle, Renderer, Vector2};
use crate::{Component, ComponentKind, Diagram};
use std::collections::{HashMap, HashSet};

const COMPONENT_HEIGHT: f64 = 50.0;
const BARRIER_WIDTH: f64 = 25.0;
const BARRIER_PADDING_RIGHT: f64 = 10.0;
const COMPONENT_MARGIN_BOTTOM: f64 = 20.0;
const COMPONENT_PADDING_X: f64 = 10.0;
const BARRIER_MARGIN_RIGHT: f64 = 50.0;
const BARRIERS_CONTAINER_HORIZONTAL_PADDING: f64 = 150.0;

pub(crate) struct Brush<'d> {
    context: Context,
    diagram: &'d Diagram,
    causes: Vec<&'d Component>,
    consequences: Vec<&'d Component>,
}

/// Holds state variables for rendering purposes.
struct Context {
    canvas_height: f64,
    canvas_width: f64,
    causes_container_height: f64,
    consequences_container_height: f64,
    max_component_box_width: f64,
    circle_left_point: Option<Vector2>,
    circle_right_point: Option<Vector2>,
}

impl<'d> Brush<'d> {
    pub fn render_diagram_into_bytes<R>(r: R, diagram: &'d Diagram) -> Vec<u8>
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
        let (r, context) = setup_canvas(
            r,
            &causes,
            &consequences,
            diagram,
            max_component_box_width,
            max_barrier_container_width,
        );
        let mut brush = Brush {
            diagram,
            context,
            causes,
            consequences,
        };
        brush.render(r)
    }

    fn render<R>(&mut self, mut r: R) -> Vec<u8>
    where
        R: Renderer,
    {
        // Draw a border around the canvas, mostly for debugging purposes.
        r = r.draw_rectangle(&Rectangle {
            centre: Vector2 {
                x: self.context.canvas_width / 2.0,
                y: self.context.canvas_height / 2.0,
            },
            width: self.context.canvas_width,
            height: self.context.canvas_height,
        });
        r = self.render_components(r, ComponentKind::Cause);
        r = self.render_components(r, ComponentKind::Consequence);
        r = self.render_event_circle(r);
        r = self.render_barrier_lines(r, ComponentKind::Cause);
        r = self.render_barrier_lines(r, ComponentKind::Consequence);
        r = self.render_barriers(r, ComponentKind::Cause, 0);
        r = self.render_barriers(r, ComponentKind::Consequence, self.causes.len());
        r.into_bytes()
    }

    fn render_event_circle<R>(&mut self, mut r: R) -> R
    where
        R: Renderer,
    {
        let radius = calculate_event_circle_radius(&self.diagram.event);
        r = r.draw_circle(
            radius,
            &Vector2 {
                x: self.context.canvas_width / 2.0,
                y: self.context.canvas_height / 2.0,
            },
        );
        r = r.draw_text(
            &self.diagram.event,
            &Rectangle {
                centre: Vector2 {
                    x: self.context.canvas_width / 2.0,
                    y: self.context.canvas_height / 2.0,
                },
                width: radius,
                height: radius,
            },
            Alignment::Center,
        );
        self.context.circle_left_point = Some(Vector2 {
            x: self.context.canvas_width / 2.0 - radius,
            y: self.context.canvas_height / 2.0,
        });
        self.context.circle_right_point = Some(Vector2 {
            x: self.context.canvas_width / 2.0 + radius,
            y: self.context.canvas_height / 2.0,
        });
        r
    }

    fn render_components<R>(&mut self, mut r: R, kind: ComponentKind) -> R
    where
        R: Renderer,
    {
        let components = self.get_components(&kind);
        for (i, component) in components.iter().enumerate().map(|(i, c)| (i as f64, c)) {
            let y = get_component_y_center(i, &kind, &self.context);
            let x = get_component_x_center(&kind, &self.context);
            let rectangle = Rectangle {
                centre: Vector2 { x, y },
                width: self.context.max_component_box_width,
                height: COMPONENT_HEIGHT,
            };
            r = r.draw_text_with_rectangle(&component.name, &rectangle, Alignment::Center);
        }
        r
    }

    fn render_barrier_lines<R>(&mut self, mut r: R, kind: ComponentKind) -> R
    where
        R: Renderer,
    {
        let components = self.get_components(&kind);
        let circle_point = self.get_component_circle_point(&kind);
        for (i, _) in components.into_iter().enumerate() {
            r = r.draw_line(&self.get_component_edge(&kind, i), &circle_point);
        }
        r
    }

    fn get_component_edge(&self, kind: &ComponentKind, i: usize) -> Vector2 {
        let y = get_component_y_center(i as f64, &kind, &self.context);
        let x_center = get_component_x_center(&kind, &self.context);
        let x_edge = match kind {
            ComponentKind::Cause => x_center + self.context.max_component_box_width / 2.0,
            ComponentKind::Consequence => x_center - self.context.max_component_box_width / 2.0,
        };
        Vector2 { x: x_edge, y }
    }

    fn render_barriers<R>(&mut self, mut r: R, kind: ComponentKind, id_offset: usize) -> R
    where
        R: Renderer,
    {
        let components = self.get_components(&kind);
        let circle_point = self.get_component_circle_point(&kind);
        let barrier_frequencies = get_barrier_frequencies(components)
            .into_iter()
            .map(|(barrier, _)| barrier);
        for (i, barrier) in barrier_frequencies.enumerate() {
            let x = get_barrier_x_center(i as f64, &kind, &self.context);
            let label_id = format!("{}", id_offset + i + 1);
            r = r.draw_text(
                &label_id,
                &Rectangle {
                    centre: Vector2 {
                        x,
                        y: get_component_y_center(-1.0, &kind, &self.context),
                    },
                    height: COMPONENT_HEIGHT,
                    width: BARRIER_WIDTH,
                },
                Alignment::Center,
            );
            let barrier_components = components.iter().enumerate().filter_map(|(j, c)| {
                if c.barriers.contains(&barrier) {
                    Some((j, c))
                } else {
                    None
                }
            });
            // Render barrier label.
            let label_y =
                get_component_y_center((components.len() + i) as f64, &kind, &self.context);
            let label_x = get_component_x_center(&kind, &self.context);
            r = r.draw_text(
                &get_barrier_label(&kind, &label_id, &barrier),
                &Rectangle {
                    centre: Vector2 {
                        y: label_y,
                        x: label_x,
                    },
                    width: self.context.max_component_box_width,
                    height: COMPONENT_HEIGHT,
                },
                get_barrier_label_alignment(&kind),
            );
            for (j, _) in barrier_components {
                let barrier_point =
                    get_slope_point(&self.get_component_edge(&kind, j), &circle_point, x);
                // Render barrier rectangle.
                r = r.draw_rectangle(&Rectangle {
                    centre: barrier_point,
                    height: COMPONENT_HEIGHT,
                    width: BARRIER_WIDTH,
                });
            }
        }
        r
    }

    fn get_components(&self, kind: &ComponentKind) -> &[&Component] {
        match kind {
            ComponentKind::Cause => &self.causes,
            ComponentKind::Consequence => &self.consequences,
        }
    }

    fn get_component_circle_point(&self, kind: &ComponentKind) -> Vector2 {
        match kind {
            ComponentKind::Cause => self.context.circle_left_point.unwrap().clone(),
            ComponentKind::Consequence => self.context.circle_right_point.unwrap().clone(),
        }
    }
}

fn get_barrier_frequencies(components: &[&Component]) -> Vec<(String, u32)> {
    let mut frequencies = HashMap::new();
    for component in components {
        for barrier in &component.barriers {
            let frequency = frequencies.entry(barrier.clone()).or_insert(0);
            *frequency += 1;
        }
    }
    let mut frequencies = frequencies.into_iter().collect::<Vec<(_, _)>>();
    frequencies.sort_by(|a, b| a.1.cmp(&b.1).reverse());
    frequencies
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
) -> (R, Context)
where
    R: Renderer,
{
    let causes_container_height = calculate_components_container_height(causes);
    let consequences_container_height = calculate_components_container_height(consequences);
    let max_barriers_height =
        calculate_barriers_height(causes) + calculate_barriers_height(consequences);
    let max_container_height =
        causes_container_height.max(consequences_container_height) + max_barriers_height;
    let canvas_height = max_container_height * 1.1 + 150.0;
    let canvas_width = calculate_canvas_width(
        diagram,
        max_component_box_width,
        max_barriers_container_width,
    );
    let canvas = Context {
        canvas_height,
        canvas_width,
        causes_container_height,
        consequences_container_height,
        max_component_box_width,
        circle_left_point: None,
        circle_right_point: None,
    };
    let r = r.setup(canvas.canvas_width, canvas.canvas_height);
    (r, canvas)
}

fn calculate_components_container_height(components: &[&Component]) -> f64 {
    let components_count = components.len() as f64;
    calculate_components_container_height_by_count(components_count)
}

fn calculate_components_container_height_by_count(components_count: f64) -> f64 {
    components_count * COMPONENT_HEIGHT + ((components_count - 1.0) * COMPONENT_MARGIN_BOTTOM)
}

fn calculate_barriers_height(components: &[&Component]) -> f64 {
    let barriers = components
        .iter()
        .flat_map(|c| c.barriers.clone())
        .collect::<HashSet<String>>()
        .len();
    calculate_components_container_height_by_count(barriers as f64)
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

fn get_component_x_center(kind: &ComponentKind, ctx: &Context) -> f64 {
    match kind {
        ComponentKind::Cause => (ctx.max_component_box_width / 2.0) + COMPONENT_PADDING_X,
        ComponentKind::Consequence => {
            ctx.canvas_width - (ctx.max_component_box_width / 2.0) - COMPONENT_PADDING_X
        }
    }
}

fn get_component_y_center(i: f64, kind: &ComponentKind, ctx: &Context) -> f64 {
    let container_height = match kind {
        ComponentKind::Cause => ctx.causes_container_height,
        ComponentKind::Consequence => ctx.consequences_container_height,
    };
    let components_container_top = (ctx.canvas_height / 2.0) - (container_height / 2.0);
    let y_relative = i * COMPONENT_HEIGHT + (i * COMPONENT_MARGIN_BOTTOM);
    components_container_top + y_relative + (COMPONENT_HEIGHT / 2.0)
}

fn get_barrier_x_center(i: f64, kind: &ComponentKind, ctx: &Context) -> f64 {
    let component_x = get_component_x_center(kind, ctx);
    match kind {
        ComponentKind::Cause => {
            component_x
                + (ctx.max_component_box_width / 2.0)
                + (i * (BARRIER_WIDTH + BARRIER_PADDING_RIGHT))
                + ((i + 1.0) * BARRIER_PADDING_RIGHT)
                + BARRIER_WIDTH / 2.0
        }
        ComponentKind::Consequence => {
            component_x
                - (ctx.max_component_box_width / 2.0)
                - (i * (BARRIER_WIDTH))
                - ((i + 1.0) * BARRIER_PADDING_RIGHT)
                - BARRIER_WIDTH / 2.0
        }
    }
}

pub fn text_width(text: &str) -> f64 {
    text.len() as f64 * 15.0
}

/// Adjusts the y-axis, given the x-axis, of a point on
/// a slope defined by `from` and `to` points.
fn get_slope_point(from: &Vector2, to: &Vector2, x: f64) -> Vector2 {
    let run = to.x - from.x;
    let rise = to.y - from.y;
    let slope = rise / run;
    let delta_x = x - from.x;
    let y = from.y + (slope * delta_x);
    Vector2 { x, y }
}

fn get_barrier_label_alignment(kind: &ComponentKind) -> Alignment {
    match kind {
        ComponentKind::Cause => Alignment::Left,
        ComponentKind::Consequence => Alignment::Right,
    }
}

fn get_barrier_label(kind: &ComponentKind, label_id: &str, barrier: &str) -> String {
    match kind {
        ComponentKind::Cause => format!("[{label_id}] {barrier}"),
        ComponentKind::Consequence => format!("{barrier} [{label_id}]"),
    }
}
