use std::{fmt, ops::Range};

use logos::Logos;

#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(skip r"[ \t\n\r\f]+")]
pub enum Token<'source> {
    #[token("and")]
    And,
    #[token("or")]
    Or,
    #[token("not")]
    Not,
    #[token("true")]
    True,
    #[token("false")]
    False,

    #[token("if")]
    If,
    #[token("then")]
    Then,
    #[token("elif")]
    Elif,
    #[token("else")]
    Else,
    #[token("for")]
    For,
    #[token("to")]
    To,
    #[token("do")]
    Do,
    #[token("break")]
    Break,
    #[token("continue")]
    Continue,
    #[token("begin")]
    Begin,
    #[token("end")]
    End,
    #[token("exit")]
    Exit,

    #[token("print")]
    Print,
    #[token("readint")]
    ReadInt,
    #[token("readstr")]
    ReadStr,
    #[token("readbool")]
    ReadBool,
    #[token("substring")]
    Substring,
    #[token("length")]
    Length,
    #[token("position")]
    Position,
    #[token("concatenate")]
    Concatenate,

    #[token("string")]
    TypeString,
    #[token("integer")]
    TypeInteger,
    #[token("boolean")]
    TypeBoolean,

    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("%")]
    Percent,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token(";")]
    Semicolon,
    #[token(",")]
    Comma,
    #[token(":=")]
    Assign,

    #[token("=")]
    NumEq,
    #[token("<")]
    Less,
    #[token("<=")]
    LessEq,
    #[token(">")]
    Greater,
    #[token(">=")]
    GreaterEq,
    #[token("<>")]
    NumNeq,

    #[token("==")]
    StrEq,
    #[token("!=")]
    StrNeq,

    #[regex(r"[0-9]+", |lex| lex.slice().parse::<i32>().ok())]
    Num(i32),

    #[regex(r#""([^"\\\x00-\x1F]|\\(["\\bnfrt/]|u[a-fA-F0-9]{4}))*""#, |lex| lex.slice())]
    StringLit(&'source str),

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice())]
    Ident(&'source str),
}

impl fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Num(n) => write!(f, "Num({})", n),
            Token::StringLit(s) => write!(f, "StringLit({})", s),
            Token::Ident(id) => write!(f, "Ident({})", id),
            other => write!(f, "{:?}", other),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LexicalError {
    InvalidToken(Range<usize>),
}

impl fmt::Display for LexicalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LexicalError::InvalidToken(range) => write!(
                f,
                "Nieprawidłowy token w zakresie {}..{}",
                range.start, range.end
            ),
        }
    }
}

pub struct Lexer<'input> {
    token_stream: logos::Lexer<'input, Token<'input>>,
}

impl<'input> Lexer<'input> {
    pub fn new(input: &'input str) -> Self {
        Lexer {
            token_stream: Token::lexer(input),
        }
    }
}

impl<'input> Iterator for Lexer<'input> {
    type Item = Result<(usize, Token<'input>, usize), LexicalError>;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.token_stream.next()?;
        let span = self.token_stream.span();

        match result {
            Ok(token) => Some(Ok((span.start, token, span.end))),
            Err(_) => Some(Err(LexicalError::InvalidToken(span))),
        }
    }
}
