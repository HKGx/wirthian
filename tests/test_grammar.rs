use interpreter::ast::{Expr, ExprKind, Program, StmtKind, Type};
use interpreter::grammar::{ExprParser, ProgramParser};
use interpreter::lexer::{Lexer, LexicalError};
use lalrpop_util::ParseError;

fn parse_program<'a>(input: &'a str, arena: &'a bumpalo::Bump) -> Program<'a> {
    ProgramParser::new()
        .parse(input, arena, Lexer::new(input))
        .unwrap()
}

fn parse_expr<'a>(input: &'a str, arena: &'a bumpalo::Bump) -> &'a Expr<'a> {
    ExprParser::new().parse(input, arena, Lexer::new(input)).unwrap()
}

fn assert_ident(expr: &Expr<'_>, expected: &str) {
    assert_eq!(expr.kind, ExprKind::Identifier(expected));
}

fn assert_num(expr: &Expr<'_>, expected: i32) {
    assert_eq!(expr.kind, ExprKind::Number(expected));
}

#[test]
fn test_empty_program() {
    let arena = bumpalo::Bump::new();
    let program = parse_program("", &arena);

    assert!(program.declarations.is_empty());
    assert!(program.instructions.is_empty());
}

#[test]
fn test_declarations_and_assignments() {
    let arena = bumpalo::Bump::new();
    let program = parse_program(
        r#"
        integer x;
        string name;
        boolean ready;

        x := 42;
        name := "Ada";
        ready := true;
        "#,
        &arena,
    );

    assert_eq!(program.declarations.len(), 3);
    assert_eq!(program.declarations[0].var_type, Type::Integer);
    assert_eq!(program.declarations[0].identifier, "x");
    assert_eq!(program.declarations[1].var_type, Type::String);
    assert_eq!(program.declarations[1].identifier, "name");
    assert_eq!(program.declarations[2].var_type, Type::Boolean);
    assert_eq!(program.declarations[2].identifier, "ready");

    assert_eq!(program.instructions.len(), 3);
    match &program.instructions[0].kind {
        StmtKind::Assign(id, expr) => {
            assert_eq!(*id, "x");
            assert_num(expr, 42);
        }
        other => panic!("expected integer assignment, got {other:?}"),
    }
    match &program.instructions[1].kind {
        StmtKind::Assign(id, expr) => {
            assert_eq!(*id, "name");
            assert_eq!(expr.kind, ExprKind::StringLit(r#""Ada""#));
        }
        other => panic!("expected string assignment, got {other:?}"),
    }
    match &program.instructions[2].kind {
        StmtKind::Assign(id, expr) => {
            assert_eq!(*id, "ready");
            assert_eq!(expr.kind, ExprKind::True);
        }
        other => panic!("expected boolean assignment, got {other:?}"),
    }
}

#[test]
fn test_numeric_expression_precedence_and_associativity() {
    let arena = bumpalo::Bump::new();
    let expr = parse_expr("1 + 2 * 3 - 4 / 2 % 5", &arena);

    match &expr.kind {
        ExprKind::Sub(left, right) => {
            match &left.kind {
                ExprKind::Add(add_left, add_right) => {
                    assert_num(add_left, 1);
                    match &add_right.kind {
                        ExprKind::Mul(mul_left, mul_right) => {
                            assert_num(mul_left, 2);
                            assert_num(mul_right, 3);
                        }
                        other => {
                            panic!("expected multiplication on right side of add, got {other:?}")
                        }
                    }
                }
                other => panic!("expected addition on left side of subtract, got {other:?}"),
            }

            match &right.kind {
                ExprKind::Mod(mod_left, mod_right) => {
                    match &mod_left.kind {
                        ExprKind::Div(div_left, div_right) => {
                            assert_num(div_left, 4);
                            assert_num(div_right, 2);
                        }
                        other => panic!("expected division on left side of modulo, got {other:?}"),
                    }
                    assert_num(mod_right, 5);
                }
                other => panic!("expected modulo on right side of subtract, got {other:?}"),
            }
        }
        other => panic!("expected top-level subtraction, got {other:?}"),
    }
}

#[test]
fn test_unary_minus_and_parenthesized_numeric_expression() {
    let arena = bumpalo::Bump::new();
    let expr = parse_expr("-(1 + 2) * -x", &arena);

    match &expr.kind {
        ExprKind::Mul(left, right) => {
            match &left.kind {
                ExprKind::Sub(zero, grouped) => {
                    assert_num(zero, 0);
                    assert!(matches!(grouped.kind, ExprKind::Add(_, _)));
                }
                other => panic!("expected unary minus on left operand, got {other:?}"),
            }
            match &right.kind {
                ExprKind::Sub(zero, id) => {
                    assert_num(zero, 0);
                    assert_ident(id, "x");
                }
                other => panic!("expected unary minus on right operand, got {other:?}"),
            }
        }
        other => panic!("expected multiplication, got {other:?}"),
    }
}

#[test]
fn test_string_and_numeric_builtin_expressions() {
    let arena = bumpalo::Bump::new();
    let program = parse_program(
        r#"
        s := concatenate(readstr, substring("abcdef", 1, 3));
        n := length(s) + position("b", s);
        "#,
        &arena,
    );

    match &program.instructions[0].kind {
        StmtKind::Assign(id, expr) => {
            assert_eq!(*id, "s");
            match &expr.kind {
                ExprKind::Concatenate(left, right) => {
                    assert_eq!(left.kind, ExprKind::ReadStr);
                    assert!(matches!(right.kind, ExprKind::Substring(_, _, _)));
                }
                other => panic!("expected concatenate expression, got {other:?}"),
            }
        }
        other => panic!("expected assignment, got {other:?}"),
    }

    match &program.instructions[1].kind {
        StmtKind::Assign(id, expr) => {
            assert_eq!(*id, "n");
            match &expr.kind {
                ExprKind::Add(left, right) => {
                    assert!(matches!(left.kind, ExprKind::Length(_)));
                    assert!(matches!(right.kind, ExprKind::Position(_, _)));
                }
                other => panic!("expected length + position expression, got {other:?}"),
            }
        }
        other => panic!("expected assignment, got {other:?}"),
    }
}

#[test]
fn test_boolean_precedence_and_comparisons() {
    let arena = bumpalo::Bump::new();
    let expr = parse_expr(r#"not false or 1 + 2 * 3 <= 7 and "a" != readstr"#, &arena);

    match &expr.kind {
        ExprKind::Or(left, right) => {
            assert!(matches!(left.kind, ExprKind::Not(_)));
            match &right.kind {
                ExprKind::And(and_left, and_right) => {
                    assert!(matches!(and_left.kind, ExprKind::LessEq(_, _)));
                    assert!(matches!(and_right.kind, ExprKind::Neq(_, _)));
                }
                other => panic!("expected and on right side of or, got {other:?}"),
            }
        }
        other => panic!("expected top-level or expression, got {other:?}"),
    }
}

#[test]
fn test_blocks_loops_and_control_flow_statements() {
    let arena = bumpalo::Bump::new();
    let program = parse_program(
        r#"
        for i := 1 to 10 do begin
            print(i);
            continue;
            break;
            exit;
        end;
        "#,
        &arena,
    );

    assert_eq!(program.instructions.len(), 1);
    match &program.instructions[0].kind {
        StmtKind::For {
            iterator,
            from,
            to,
            body,
        } => {
            assert_eq!(*iterator, "i");
            assert_num(from, 1);
            assert_num(to, 10);
            match &body.kind {
                StmtKind::Block(statements) => {
                    assert_eq!(statements.len(), 4);
                    assert!(matches!(statements[0].kind, StmtKind::Print(_)));
                    assert!(matches!(statements[1].kind, StmtKind::Continue));
                    assert!(matches!(statements[2].kind, StmtKind::Break));
                    assert!(matches!(statements[3].kind, StmtKind::Exit));
                }
                other => panic!("expected for body block, got {other:?}"),
            }
        }
        other => panic!("expected for statement, got {other:?}"),
    }
}

#[test]
fn test_if_elif_else_tree() {
    let arena = bumpalo::Bump::new();
    let program = parse_program(
        r#"
        if x = 0 then print("zero")
        elif x = 1 then print("one")
        elif x = 2 then print("two")
        else print("many");
        "#,
        &arena,
    );

    assert_eq!(program.instructions.len(), 1);
    match &program.instructions[0].kind {
        StmtKind::If {
            condition,
            then_branch,
            elif_branches,
            else_branch,
        } => {
            assert!(matches!(condition.kind, ExprKind::Eq(_, _)));
            assert!(matches!(then_branch.kind, StmtKind::Print(_)));
            assert_eq!(elif_branches.len(), 2);
            assert!(matches!(elif_branches[0].0.kind, ExprKind::Eq(_, _)));
            assert!(matches!(elif_branches[0].1.kind, StmtKind::Print(_)));
            assert!(matches!(elif_branches[1].0.kind, ExprKind::Eq(_, _)));
            assert!(matches!(elif_branches[1].1.kind, StmtKind::Print(_)));
            assert!(matches!(
                else_branch.map(|stmt| &stmt.kind),
                Some(StmtKind::Print(_))
            ));
        }
        other => panic!("expected if statement, got {other:?}"),
    }
}

#[test]
fn test_dangling_else_binds_to_nearest_if() {
    let arena = bumpalo::Bump::new();
    let program = parse_program(
        r#"
        if outer then
            if inner then print("inner")
            else print("else");
        "#,
        &arena,
    );

    match &program.instructions[0].kind {
        StmtKind::If {
            else_branch,
            then_branch,
            ..
        } => {
            assert!(
                else_branch.is_none(),
                "outer if should not receive the else branch"
            );
            match &then_branch.kind {
                StmtKind::If {
                    else_branch: inner_else,
                    ..
                } => {
                    assert!(
                        inner_else.is_some(),
                        "inner if should receive the else branch"
                    );
                }
                other => panic!("expected nested if as then branch, got {other:?}"),
            }
        }
        other => panic!("expected outer if statement, got {other:?}"),
    }
}

#[test]
fn test_program_parser_reports_lexical_errors() {
    let arena = bumpalo::Bump::new();
    let err = ProgramParser::new()
        .parse("x := @;", &arena, Lexer::new("x := @;"))
        .unwrap_err();

    assert!(matches!(
        err,
        ParseError::User { error: LexicalError::InvalidToken(range) } if range == (5..6)
    ));
}

#[test]
fn test_program_parser_rejects_missing_semicolon() {
    let arena = bumpalo::Bump::new();
    let err = ProgramParser::new()
        .parse("integer x x := 1;", &arena, Lexer::new("integer x x := 1;"))
        .unwrap_err();

    assert!(matches!(err, ParseError::UnrecognizedToken { .. }));
}
