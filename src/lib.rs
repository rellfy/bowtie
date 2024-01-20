#[derive(Default, Debug)]
struct Diagram {
    title: String,
    event: String,
    components: Vec<Component>,
}

#[derive(Debug)]
struct Component {
    name: String,
    barriers: Vec<String>,
    kind: ComponentKind,
}

#[derive(Debug, Eq, PartialEq)]
enum ComponentKind {
    Cause,
    Consequence,
}

pub fn generate_bowtie(input: &str) -> Vec<u8> {
    let diagram = parse_diagram(input);
    println!("diagram: {:#?}", diagram);
    vec![]
}

fn parse_diagram(input: &str) -> Diagram {
    let mut diagram = Diagram::default();
    let lines = input.lines();
    for line in lines {
        let Some((command, value)) = line.split_once(' ') else {
            continue;
        };
        let value = value.trim();
        match command {
            "title" => {
                diagram.title = value.to_owned();
            }
            "cause" => {
                let is_new = !diagram
                    .components
                    .iter()
                    .any(|c| c.name == value && c.kind == ComponentKind::Cause);
                if !is_new {
                    continue;
                }
                let component = Component {
                    name: value.to_owned(),
                    barriers: Vec::new(),
                    kind: ComponentKind::Cause,
                };
                diagram.components.push(component);
            }
            "consequence" => {
                let is_new = !diagram
                    .components
                    .iter()
                    .any(|c| c.name == value && c.kind == ComponentKind::Consequence);
                if !is_new {
                    continue;
                }
                let component = Component {
                    name: value.to_owned(),
                    barriers: Vec::new(),
                    kind: ComponentKind::Consequence,
                };
                diagram.components.push(component);
            }
            "event" => {
                diagram.event = value.to_owned();
            }
            "barrier" => {
                let Some((barrier_name, components_name)) = value.split_once(':') else {
                    continue;
                };
                let barrier_name = barrier_name.trim();
                let component_names = components_name.trim().split(',').collect::<Vec<_>>();
                let components = diagram
                    .components
                    .iter_mut()
                    .filter(|c| component_names.iter().any(|name| c.name == name.trim()));
                for component in components {
                    component.barriers.push(barrier_name.to_owned());
                }
            }
            _ => {
                continue;
            }
        }
    }
    diagram
}
