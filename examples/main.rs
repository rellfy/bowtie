use bowtie::generate_bowtie;

const INPUT: &str = include_str!("./chemical_spillage.txt");

fn main() {
    println!("generating diagram...");
    let svg_bytes = generate_bowtie(INPUT);
    std::fs::write("./chemical_spillage.svg", svg_bytes).unwrap();
    println!("written to chemical_spillage.svg");
}
