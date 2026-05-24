use divan::{Bencher, black_box, counter::BytesCount};
use interpreter::{grammar::ProgramParser, lexer::Lexer};

mod generate;

fn main() {
    divan::main();
}

fn parse_program(input: &str) {
    let lexer = Lexer::new(input);
    let parser = ProgramParser::new();
    let ast = parser
        .parse(input, lexer)
        .expect("generated benchmark input must parse successfully");
    black_box(ast);
}

const SIZES: &[usize] = &[1024, 16 * 1024, 64 * 1024, 256 * 1024];

#[divan::bench(args = SIZES)]
fn parse(bencher: Bencher, size: usize) {
    bencher
        .with_inputs(|| generate::generated_program(size))
        .input_counter(BytesCount::of_str)
        .bench_local_refs(|a| parse_program(divan::black_box(a.as_str())));
}
