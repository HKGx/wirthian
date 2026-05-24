use interpreter::lexer::{Lexer, LexicalError, Token};

fn assert_lex<const N: usize>(input: &str, expected: [Token<'_>; N]) {
    let lexer = Lexer::new(input);
    let actual: Vec<Token<'_>> = lexer.map(|res| res.unwrap().1).collect();

    assert_eq!(actual, expected);
}

#[test]
fn test_basic_tokens() {
    assert_lex(
        "integer x; x := 10;",
        [
            Token::TypeInteger,
            Token::Ident("x"),
            Token::Semicolon,
            Token::Ident("x"),
            Token::Assign,
            Token::Num(10),
            Token::Semicolon,
        ],
    );
}

#[test]
fn test_keywords() {
    assert_lex(
        "if then elif else for to do break continue begin end exit print readint readstr readbool substring length position concatenate",
        [
            Token::If,
            Token::Then,
            Token::Elif,
            Token::Else,
            Token::For,
            Token::To,
            Token::Do,
            Token::Break,
            Token::Continue,
            Token::Begin,
            Token::End,
            Token::Exit,
            Token::Print,
            Token::ReadInt,
            Token::ReadStr,
            Token::ReadBool,
            Token::Substring,
            Token::Length,
            Token::Position,
            Token::Concatenate,
        ],
    );
}

#[test]
fn test_strings_and_nums() {
    assert_lex(
        r#"123 "hello" "with \"quotes\"""#,
        [
            Token::Num(123),
            Token::StringLit(r#""hello""#),
            Token::StringLit(r#""with \"quotes\"""#),
        ],
    );
}

#[test]
fn test_whitespace_and_newlines() {
    assert_lex(
        " \t\n\r  integer \n\n x \t ; \n",
        [Token::TypeInteger, Token::Ident("x"), Token::Semicolon],
    );
}

#[test]
fn test_complex_identifiers() {
    assert_lex(
        "_myVar123 another_var X y1",
        [
            Token::Ident("_myVar123"),
            Token::Ident("another_var"),
            Token::Ident("X"),
            Token::Ident("y1"),
        ],
    );
}

#[test]
fn test_operators_and_greedy_matching() {
    assert_lex(
        "x<=10 a==b y!=z",
        [
            Token::Ident("x"),
            Token::LessEq,
            Token::Num(10),
            Token::Ident("a"),
            Token::StrEq,
            Token::Ident("b"),
            Token::Ident("y"),
            Token::StrNeq,
            Token::Ident("z"),
        ],
    );
}

#[test]
fn test_empty_input() {
    assert_lex("", []);
}

#[test]
fn test_boolean_logic_and_type_keywords() {
    assert_lex(
        "and or not true false string boolean integer",
        [
            Token::And,
            Token::Or,
            Token::Not,
            Token::True,
            Token::False,
            Token::TypeString,
            Token::TypeBoolean,
            Token::TypeInteger,
        ],
    );
}

#[test]
fn test_arithmetic_and_punctuation_tokens() {
    assert_lex(
        "+ - * / % ( ) ; , :=",
        [
            Token::Plus,
            Token::Minus,
            Token::Star,
            Token::Slash,
            Token::Percent,
            Token::LParen,
            Token::RParen,
            Token::Semicolon,
            Token::Comma,
            Token::Assign,
        ],
    );
}

#[test]
fn test_numeric_comparison_tokens() {
    assert_lex(
        "a = b a < b a <= b a > b a >= b a <> b",
        [
            Token::Ident("a"),
            Token::NumEq,
            Token::Ident("b"),
            Token::Ident("a"),
            Token::Less,
            Token::Ident("b"),
            Token::Ident("a"),
            Token::LessEq,
            Token::Ident("b"),
            Token::Ident("a"),
            Token::Greater,
            Token::Ident("b"),
            Token::Ident("a"),
            Token::GreaterEq,
            Token::Ident("b"),
            Token::Ident("a"),
            Token::NumNeq,
            Token::Ident("b"),
        ],
    );
}

#[test]
fn test_keywords_embedded_in_identifiers() {
    assert_lex(
        "ifx then_ integer1 true_value falsehood readint2",
        [
            Token::Ident("ifx"),
            Token::Ident("then_"),
            Token::Ident("integer1"),
            Token::Ident("true_value"),
            Token::Ident("falsehood"),
            Token::Ident("readint2"),
        ],
    );
}

#[test]
fn test_string_escape_sequences() {
    assert_lex(
        r#""line\n" "tab\t" "slash\\" "unicode\u0041" "quote\"""#,
        [
            Token::StringLit(r#""line\n""#),
            Token::StringLit(r#""tab\t""#),
            Token::StringLit(r#""slash\\""#),
            Token::StringLit(r#""unicode\u0041""#),
            Token::StringLit(r#""quote\"""#),
        ],
    );
}

#[test]
fn test_token_spans() {
    let actual: Vec<_> = Lexer::new("x := 42").collect();

    assert_eq!(
        actual,
        [
            Ok((0, Token::Ident("x"), 1)),
            Ok((2, Token::Assign, 4)),
            Ok((5, Token::Num(42), 7)),
        ]
    );
}

#[test]
fn test_invalid_token_error() {
    let actual: Vec<_> = Lexer::new("x @ y").collect();

    assert_eq!(
        actual,
        [
            Ok((0, Token::Ident("x"), 1)),
            Err(LexicalError::InvalidToken(2..3)),
            Ok((4, Token::Ident("y"), 5)),
        ]
    );
}
