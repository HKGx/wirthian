use divan::{Bencher, Divan};
use interpreter::{
    ast::Program, checker::Checker, grammar::ProgramParser, interp::Interpreter, lexer::Lexer,
};
use std::io::{self};

fn main() {
    Divan::from_args()
        .sample_count(30)
        .sample_size(1)
        .run_benches();
}

fn parse_static(source: &'static str) -> Program<'static> {
    let arena: &'static bumpalo::Bump = Box::leak(Box::new(bumpalo::Bump::new()));
    let lexer = Lexer::new(source);
    let parser = ProgramParser::new();

    let program = parser
        .parse(source, arena, lexer)
        .expect("benchmark program must parse");

    Checker::new()
        .check_program(&program, source)
        .expect("benchmark program must type-check");

    program
}

#[divan::bench(args = ["fib", "collatz", "primes", "mandelbrot", "strings", "fizzbuzz"])]
fn interp(bencher: Bencher, name: &str) {
    let source: &'static str = match name {
        "fib" => include_str!("../programs/fib.pa"),
        "collatz" => include_str!("../programs/collatz.pa"),
        "primes" => include_str!("../programs/primes.pa"),
        "mandelbrot" => include_str!("../programs/mandelbrot.pa"),
        "strings" => include_str!("../programs/strings.pa"),
        "fizzbuzz" => include_str!("../programs/fizzbuzz.pa"),
        _ => unreachable!(),
    };

    let program = parse_static(source);

    bencher
        .with_inputs(|| &program)
        .bench_local_refs(|program| {
            let mut interp = Interpreter::new(program, io::empty(), io::sink());
            interp.run(program).expect("run");
        });
}
