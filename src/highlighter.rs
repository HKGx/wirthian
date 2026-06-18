use miette::highlighters::{Highlighter, HighlighterState};
use owo_colors::{Style, Styled};

use crate::lexer::{Lexer, LexicalError, Token};

pub struct WirthianHighlighter;

impl Highlighter for WirthianHighlighter {
    fn start_highlighter_state<'h>(
        &'h self,
        _source: &dyn miette::SpanContents<'_>,
    ) -> Box<dyn HighlighterState + 'h> {
        Box::new(WirthianHighlighterState)
    }
}

pub struct WirthianHighlighterState;

impl HighlighterState for WirthianHighlighterState {
    fn highlight_line<'s>(&mut self, line: &'s str) -> Vec<Styled<&'s str>> {
        let default_style = Style::new();
        let mut result = Vec::new();
        let mut pos = 0;

        for item in Lexer::new(line) {
            let (start, end, style) = match item {
                Ok((start, token, end)) => (start, end, style_for_token(&token)),
                Err(LexicalError::InvalidToken(range)) => (range.start, range.end, default_style),
            };
            if start > pos {
                result.push(default_style.style(&line[pos..start]));
            }
            if end > start {
                result.push(style.style(&line[start..end]));
            }
            pos = end;
        }

        if pos < line.len() {
            result.push(default_style.style(&line[pos..]));
        }
        if result.is_empty() && !line.is_empty() {
            result.push(default_style.style(line));
        }

        result
    }
}

fn style_for_token(token: &Token<'_>) -> Style {
    let keyword_style = Style::new().bold().blue();
    let string_style = Style::new().green();
    let num_style = Style::new().yellow();

    match token {
        Token::And
        | Token::Or
        | Token::Not
        | Token::True
        | Token::False
        | Token::If
        | Token::Then
        | Token::Elif
        | Token::Else
        | Token::For
        | Token::To
        | Token::Do
        | Token::Break
        | Token::Continue
        | Token::Begin
        | Token::End
        | Token::Exit
        | Token::Print
        | Token::ReadInt
        | Token::ReadStr
        | Token::ReadBool
        | Token::Substring
        | Token::Length
        | Token::Position
        | Token::Concatenate
        | Token::TypeString
        | Token::TypeInteger
        | Token::TypeBoolean => keyword_style,

        Token::StringLit(_) => string_style,
        Token::Num(_) => num_style,
        _ => Style::new(),
    }
}
