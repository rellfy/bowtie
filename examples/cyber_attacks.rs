use bowtie::generate_bowtie;
use bowtie::renderer::SvgRenderer;

const EXAMPLE_NAME: &str = "cyber_attacks";
const INPUT: &str = include_str!("./cyber_attacks.txt");

fn main() {
    println!("generating diagram...");
    let renderer = SvgRenderer::new();
    let svg_bytes = generate_bowtie(INPUT, renderer);
    let output_file_name = format!("{EXAMPLE_NAME}.svg");
    std::fs::write(&output_file_name, svg_bytes).unwrap();
    println!("written to {output_file_name}.svg");
}
