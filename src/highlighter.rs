use miette::highlighters::{Highlighter, HighlighterState};
use owo_colors::{Style, Styled};

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
        let keywords = [
            "integer", "string", "if", "then", "else", "for", "print", 
            "concatenate", "substring", "length", "position", 
            "exit", "break", "continue", "true", "false", "to", "do", "elif", "boolean"
        ];
        
        let mut result = Vec::new();
        let mut last_idx = 0;
        
        for (i, _) in line.match_indices(|c: char| !c.is_alphanumeric() && c != '_') {
            if i > last_idx {
                let word = &line[last_idx..i];
                if keywords.contains(&word) {
                    result.push(Style::new().bold().blue().style(word));
                } else {
                    result.push(Style::new().style(word));
                }
            }
            let sep = &line[i..i+1];
            result.push(Style::new().style(sep));
            last_idx = i + 1;
        }
        
        if last_idx < line.len() {
            let word = &line[last_idx..];
            if keywords.contains(&word) {
                result.push(Style::new().bold().blue().style(word));
            } else {
                result.push(Style::new().style(word));
            }
        }
        
        if result.is_empty() && !line.is_empty() {
             result.push(Style::new().style(line));
        }

        result
    }
}