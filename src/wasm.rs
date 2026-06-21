use crate::{checker, grammar, interp, lexer};
use bumpalo::Bump;
use lalrpop_util::ParseError;
use serde::Serialize;
use std::io::{BufReader, Cursor};
use std::ops::Range;
use wasm_bindgen::prelude::*;

#[derive(Serialize)]
struct SourceDiagnostic {
    message: String,
    from: usize,
    to: usize,
    severity: &'static str,
}

#[wasm_bindgen]
pub struct RunResult {
    output: String,
    error: Option<String>,
    diagnostics: Vec<SourceDiagnostic>,
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

    #[wasm_bindgen(getter)]
    pub fn diagnostics(&self) -> Result<JsValue, JsValue> {
        serde_wasm_bindgen::to_value(&self.diagnostics).map_err(Into::into)
    }
}

fn editor_offset(source: &str, byte_offset: usize) -> usize {
    source[..byte_offset].encode_utf16().count()
}

fn diagnostic(
    source: &str,
    span: Range<usize>,
    message: impl Into<String>,
    severity: &'static str,
) -> SourceDiagnostic {
    SourceDiagnostic {
        message: message.into(),
        from: editor_offset(source, span.start),
        to: editor_offset(source, span.end),
        severity,
    }
}

fn parse_diagnostic(
    source: &str,
    error: ParseError<usize, lexer::Token<'_>, lexer::LexicalError>,
) -> SourceDiagnostic {
    match error {
        ParseError::InvalidToken { location } => diagnostic(
            source,
            location..location + 1,
            "Nieprawidłowy token",
            "error",
        ),
        ParseError::UnrecognizedEof { location, expected } => diagnostic(
            source,
            location..location,
            format!(
                "Nieoczekiwany koniec kodu. Oczekiwano: {}",
                expected.join(", ")
            ),
            "error",
        ),
        ParseError::UnrecognizedToken {
            token: (from, token, to),
            expected,
        } => diagnostic(
            source,
            from..to,
            format!(
                "Nieoczekiwany token '{token}'. Oczekiwano: {}",
                expected.join(", ")
            ),
            "error",
        ),
        ParseError::ExtraToken {
            token: (from, token, to),
        } => diagnostic(
            source,
            from..to,
            format!("Nadmiarowy token '{token}'"),
            "error",
        ),
        ParseError::User {
            error: lexer::LexicalError::InvalidToken(span),
        } => diagnostic(source, span, "Nieprawidłowy token", "error"),
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
                error: None,
                diagnostics: vec![parse_diagnostic(source, e)],
            };
        }
    };

    if let Err(report) = checker::Checker::new().check_program(&ast, source) {
        let diagnostics = report
            .errors
            .into_iter()
            .flat_map(|error| {
                error.labels.into_iter().map(move |label| {
                    let message = label.label().unwrap_or(&error.message).to_owned();
                    let severity = if label.primary() { "error" } else { "info" };
                    diagnostic(
                        source,
                        label.offset()..label.offset() + label.len(),
                        message,
                        severity,
                    )
                })
            })
            .collect();
        return RunResult {
            output: String::new(),
            error: None,
            diagnostics,
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
                diagnostics: Vec::new(),
            };
        }
        let _ = interpreter.flush();
    }

    RunResult {
        output: String::from_utf8_lossy(&out).into_owned(),
        error: None,
        diagnostics: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semantic_errors_include_primary_and_related_labels() {
        let source = "integer i; string s; i := s;";
        let result = run_source(source, "");

        assert_eq!(result.diagnostics.len(), 3);
        assert_eq!(
            result.diagnostics[0].message,
            "Niezgodność typów: przypisujesz wartość typu String do zmiennej 'i' typu Integer"
        );
        assert_eq!(result.diagnostics[0].severity, "error");
        let marked_source: Vec<_> = result
            .diagnostics
            .iter()
            .map(|diagnostic| &source[diagnostic.from..diagnostic.to])
            .collect();
        assert_eq!(marked_source, ["s", "integer i", "i := s"]);
        assert!(
            result.diagnostics[1..]
                .iter()
                .all(|diagnostic| diagnostic.severity == "info")
        );
    }

    #[test]
    fn source_offsets_are_converted_to_utf16_positions() {
        assert_eq!(editor_offset("ąx", "ą".len()), 1);
    }
}
