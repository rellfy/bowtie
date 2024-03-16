use bowtie::generate_bowtie;
use bowtie::renderer::SvgRenderer;

const INPUT: &str = include_str!("./chemical_spillage.txt");

fn main() {
    println!("generating diagram...");
    let renderer = SvgRenderer::new();
    let svg_bytes = generate_bowtie(INPUT, renderer);
    std::fs::write("./chemical_spillage.svg", svg_bytes).unwrap();
    println!("written to chemical_spillage.svg");
}
