use bowtie::generate_bowtie;
use bowtie::renderer::SvgRenderer;

const EXAMPLE_NAME: &str = "chemical_spillage";
const INPUT: &str = include_str!("./chemical_spillage.txt");

fn main() {
    println!("generating diagram...");
    let renderer = SvgRenderer::new();
    let svg_bytes = generate_bowtie(INPUT, renderer);
    let output_file_name = format!("{EXAMPLE_NAME}.svg");
    std::fs::write(&output_file_name, svg_bytes).unwrap();
    println!("written to {output_file_name}.svg");
}
