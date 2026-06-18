use divan::{Bencher, black_box, counter::BytesCount};
use interpreter::lexer::Lexer;

mod generate;

fn main() {
    divan::main();
}

fn lex_all(input: &str) -> usize {
    Lexer::new(input)
        .map(|result| {
            let (_, token, _) = result.expect("generated benchmark input must lex successfully");
            black_box(token);
            1usize
        })
        .sum()
}

const SIZES: &[usize] = &[1024, 16 * 1024, 64 * 1024, 256 * 1024, 1024 * 1024];

#[divan::bench(args = SIZES)]
fn lex(bencher: Bencher, size: usize) {
    bencher
        .with_inputs(|| generate::generated_program(size))
        .input_counter(BytesCount::of_str)
        .bench_local_refs(|a| lex_all(divan::black_box(a.as_str())));
}
