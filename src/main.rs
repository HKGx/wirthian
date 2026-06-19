use interpreter::{checker, grammar, highlighter, lexer};
use miette::{MietteHandlerOpts, Report};

fn main() -> miette::Result<()> {
    miette::set_hook(Box::new(|_| {
        Box::new(
            MietteHandlerOpts::new()
                .with_syntax_highlighting(highlighter::WirthianHighlighter)
                .build(),
        )
    }))
    .expect("failed to set miette hook");

    let source_code = "
    integer x;
    string y;

    x := 10;
    x := concatenate(\"A\", \"B\");
    y := substring(x, 1, 2);

    if x > 5 then
        print(x);
    ";

    let arena = bumpalo::Bump::new();
    let lexer = lexer::Lexer::new(source_code);
    let parser = grammar::ProgramParser::new();

    match parser.parse(source_code, &arena, lexer) {
        Ok(ast) => {
            let checker = checker::Checker::new();
            match checker.check_program(&ast, source_code) {
                Ok(_) => {
                    println!("Analiza semantyczna zakończona sukcesem.");
                    println!("Program nie zawiera błędów typowania!");
                }
                Err(report) => {
                    eprintln!("ZNALEZIONO BŁĘDY SEMANTYCZNE\n");
                    for err in report.errors {
                        eprintln!("{:?}", Report::new(err));
                    }
                }
            }
        }
        Err(e) => {
            // For parsing errors, we can also use miette if we wrap them
            // But for now let's just print them
            println!("Błąd parsowania: {:?}", e);
        }
    }

    Ok(())
}
