use crate::{checker, grammar, interp, lexer};
use bumpalo::Bump;
use std::io::{BufReader, Cursor};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct RunResult {
    output: String,
    error: Option<String>,
}

#[wasm_bindgen]
impl RunResult {
    #[wasm_bindgen(getter)]
    pub fn output(&self) -> String {
        self.output.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn error(&self) -> Option<String> {
        self.error.clone()
    }
}

#[wasm_bindgen]
pub fn run_source(source: &str, input: &str) -> RunResult {
    console_error_panic_hook::set_once();

    let arena = Bump::new();
    let lex = lexer::Lexer::new(source);
    let parser = grammar::ProgramParser::new();

    let ast = match parser.parse(source, &arena, lex) {
        Ok(ast) => ast,
        Err(e) => {
            return RunResult {
                output: String::new(),
                error: Some(format!("Błąd parsowania: {e:?}")),
            }
        }
    };

    if let Err(report) = checker::Checker::new().check_program(&ast, source) {
        let msgs = report
            .errors
            .into_iter()
            .map(|e| format!("{:?}", miette::Report::new(e)))
            .collect::<Vec<_>>()
            .join("\n\n");
        return RunResult {
            output: String::new(),
            error: Some(msgs),
        };
    }

    let mut out: Vec<u8> = Vec::new();
    {
        let input_reader = BufReader::new(Cursor::new(input.as_bytes()));
        let mut interpreter = interp::Interpreter::new(&ast, input_reader, &mut out);
        if let Err(e) = interpreter.run(&ast) {
            let _ = interpreter.flush();
            return RunResult {
                output: String::from_utf8_lossy(&out).into_owned(),
                error: Some(format!("Błąd wykonania: {e}")),
            };
        }
        let _ = interpreter.flush();
    }

    RunResult {
        output: String::from_utf8_lossy(&out).into_owned(),
        error: None,
    }
}
