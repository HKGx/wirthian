use interpreter::{checker, grammar, highlighter, interp, lexer};
use miette::{MietteHandlerOpts, Report};
use std::fs;
use std::io::{self, BufReader, BufWriter};
use std::process::ExitCode;

fn main() -> ExitCode {
    miette::set_hook(Box::new(|_| {
        Box::new(
            MietteHandlerOpts::new()
                .with_syntax_highlighting(highlighter::WirthianHighlighter)
                .build(),
        )
    }))
    .expect("failed to set miette hook");

    let mut args = std::env::args().skip(1);
    let path = match (args.next().as_deref(), args.next()) {
        (Some("run"), Some(file)) => file,
        (Some(file), _) if file != "run" => file.to_string(),
        _ => {
            eprintln!("Usage: interpreter run <file.pa>");
            return ExitCode::from(2);
        }
    };

    let source = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Nie można odczytać pliku '{path}': {e}");
            return ExitCode::from(2);
        }
    };

    let arena = bumpalo::Bump::new();
    let lex = lexer::Lexer::new(&source);
    let parser = grammar::ProgramParser::new();

    let ast = match parser.parse(&source, &arena, lex) {
        Ok(ast) => ast,
        Err(e) => {
            println!("Błąd parsowania: {e:?}");
            return ExitCode::from(1);
        }
    };

    if let Err(report) = checker::Checker::new().check_program(&ast, &source) {
        eprintln!("ZNALEZIONO BŁĘDY SEMANTYCZNE\n");
        for err in report.errors {
            eprintln!("{:?}", Report::new(err));
        }
        return ExitCode::from(1);
    }

    let stdin = BufReader::new(io::stdin());
    let stdout = io::stdout();
    let mut interp = interp::Interpreter::new(&ast, stdin, BufWriter::new(stdout.lock()));
    if let Err(e) = interp.run(&ast) {
        let _ = interp.flush();
        eprintln!("Błąd wykonania: {e}");
        return ExitCode::from(1);
    }
    let _ = interp.flush();
    ExitCode::SUCCESS
}
