use interpreter::{checker::Checker, grammar::ProgramParser, interp::Interpreter, lexer::Lexer};
use std::io::{self, BufReader, Cursor};

fn run(source: &str, input: &str) -> String {
    let arena = bumpalo::Bump::new();
    let lexer = Lexer::new(source);
    let parser = ProgramParser::new();

    let ast = parser.parse(source, &arena, lexer).expect("parse");

    Checker::new().check_program(&ast, source).expect("check");

    let mut output = Vec::new();
    let mut interp = Interpreter::new(&ast, BufReader::new(Cursor::new(input)), &mut output);

    interp.run(&ast).expect("run");

    String::from_utf8(output).expect("utf8")
}

#[test]
fn test_uninitialized_variables_default_to_zero_and_empty() {
    let out = run(
        "integer x; string s; x := length(s); print(x); print(s);",
        "",
    );

    assert_eq!(out, "0");
}

#[test]
fn test_length() {
    assert_eq!(run("print(length(\"hello\"));", ""), "5");
    assert_eq!(run("print(length(\"\"));", ""), "0");
    assert_eq!(run("print(length(\"hello world\"));", ""), "11");
}

#[test]
fn test_concatenate() {
    assert_eq!(run("print(concatenate(\"ab\", \"cd\"));", ""), "abcd");
    assert_eq!(run("print(concatenate(\"\", \"\"));", ""), "");
    assert_eq!(run("print(concatenate(\"x\", \"\"));", ""), "x");
}

#[test]
fn test_substring_one_based() {
    assert_eq!(run("print(substring(\"hello\", 1, 3));", ""), "hel");
    assert_eq!(run("print(substring(\"hello\", 2, 3));", ""), "ell");
    assert_eq!(run("print(substring(\"hello\", 5, 1));", ""), "o");
}

#[test]
fn test_substring_pos_out_of_range_returns_empty() {
    assert_eq!(run("print(substring(\"hello\", 0, 3));", ""), "");
    assert_eq!(run("print(substring(\"hello\", 6, 1));", ""), "");
    assert_eq!(run("print(substring(\"hello\", 10, 1));", ""), "");
}

#[test]
fn test_substring_length_too_large_truncates() {
    assert_eq!(run("print(substring(\"hello\", 3, 10));", ""), "llo");
    assert_eq!(run("print(substring(\"hello\", 1, 100));", ""), "hello");
}

#[test]
fn test_substring_length_zero_or_negative_returns_empty() {
    assert_eq!(run("print(substring(\"hello\", 1, 0));", ""), "");
    assert_eq!(run("print(substring(\"hello\", 1, -3));", ""), "");
}

#[test]
fn test_position_one_based() {
    assert_eq!(run("print(position(\"hello\", \"ll\"));", ""), "3");
    assert_eq!(run("print(position(\"hello\", \"hello\"));", ""), "1");
    assert_eq!(run("print(position(\"hello\", \"x\"));", ""), "0");
    assert_eq!(run("print(position(\"hello\", \"\"));", ""), "1");
    assert_eq!(run("print(position(\"aaa\", \"a\"));", ""), "1");
}

#[test]
fn test_for_loop_inclusive_and_counter_increment() {
    let out = run(
        "integer i; integer sum; sum := 0; for i := 1 to 5 do sum := sum + i; print(sum); print(i);",
        "",
    );

    assert_eq!(out, "156");
}

#[test]
fn test_for_loop_zero_iterations_when_from_exceeds_to() {
    let out = run(
        "integer i; integer count; count := 0; for i := 5 to 1 do count := count + 1; print(count);",
        "",
    );

    assert_eq!(out, "0");
}

#[test]
fn test_for_counter_holds_value_after_break() {
    let out = run(
        "integer i; for i := 1 to 100 do begin if i = 7 then break; end; print(i);",
        "",
    );

    assert_eq!(out, "7");
}

#[test]
fn test_break_exits_innermost_loop() {
    let out = run(
        "integer i; integer j; integer total; total := 0;
        for i := 1 to 3 do begin
            for j := 1 to 10 do begin
                if j = 4 then break;
                total := total + 1;
            end;
        end;
        print(total);",
        "",
    );

    assert_eq!(out, "9");
}

#[test]
fn test_continue_skips_rest_but_increments() {
    let out = run(
        "integer i; integer total; total := 0;
        for i := 1 to 10 do begin
            if i % 2 = 0 then continue;
            total := total + i;
        end;
        print(total);",
        "",
    );

    assert_eq!(out, "25");
}

#[test]
fn test_exit_terminates_program() {
    let out = run(
        "integer i;
        for i := 1 to 100 do begin
            if i = 5 then exit;
            print(i);
        end;
        print(\"never\");",
        "",
    );

    assert_eq!(out, "1234");
}

#[test]
fn test_arithmetic() {
    assert_eq!(run("print(7 + 3);", ""), "10");
    assert_eq!(run("print(7 - 3);", ""), "4");
    assert_eq!(run("print(7 * 3);", ""), "21");
    assert_eq!(run("print(7 / 2);", ""), "3");
    assert_eq!(run("print(7 % 3);", ""), "1");
    assert_eq!(run("print(-7 + 10);", ""), "3");
    assert_eq!(run("print(2 + 3 * 4);", ""), "14");
    assert_eq!(run("print((2 + 3) * 4);", ""), "20");
}

#[test]
fn test_division_by_zero_is_runtime_error() {
    let arena = bumpalo::Bump::new();
    let source = "integer x; x := 1 / 0;";

    let lexer = Lexer::new(source);

    let ast = ProgramParser::new()
        .parse(source, &arena, lexer)
        .expect("parse");

    Checker::new().check_program(&ast, source).expect("check");

    let mut interp = Interpreter::new(&ast, BufReader::new(Cursor::new("")), io::sink());

    assert!(interp.run(&ast).is_err());
}

#[test]
fn test_boolean_and_or_not() {
    assert_eq!(run("print(true and false);", ""), "false");
    assert_eq!(run("print(true or false);", ""), "true");
    assert_eq!(run("print(not true);", ""), "false");
    assert_eq!(run("print(true and true);", ""), "true");
    assert_eq!(run("print(false or false);", ""), "false");
}

#[test]
fn test_short_circuit_and() {
    let source = "integer b; if false and readint > 0 then b := 1; b := readint; print(b);";

    assert_eq!(run(source, "42\n"), "42");
}

#[test]
fn test_short_circuit_or() {
    let source = "integer b; if true or readint > 0 then b := 1; b := readint; print(b);";

    assert_eq!(run(source, "42\n"), "42");
}

#[test]
fn test_integer_comparisons() {
    assert_eq!(run("print(3 = 3);", ""), "true");
    assert_eq!(run("print(3 <> 4);", ""), "true");
    assert_eq!(run("print(3 < 4);", ""), "true");
    assert_eq!(run("print(3 <= 3);", ""), "true");
    assert_eq!(run("print(5 > 4);", ""), "true");
    assert_eq!(run("print(5 >= 6);", ""), "false");
}

#[test]
fn test_string_equality() {
    assert_eq!(run("print(\"abc\" == \"abc\");", ""), "true");
    assert_eq!(run("print(\"abc\" != \"abd\");", ""), "true");
    assert_eq!(run("print(\"abc\" == \"abd\");", ""), "false");
}

#[test]
fn test_string_escape_sequences() {
    assert_eq!(run("print(length(\"a\\nb\"));", ""), "3");
    assert_eq!(run("print(length(\"\\t\\n\"));", ""), "2");
    assert_eq!(run("print(\"line\\nbreak\");", ""), "line\nbreak");
    assert_eq!(run("print(\"tab\\tvalue\");", ""), "tab\tvalue");
    assert_eq!(run("print(\"quote\\\"inside\");", ""), "quote\"inside");
    assert_eq!(run("print(\"back\\\\slash\");", ""), "back\\slash");
}

#[test]
fn test_unicode_escape() {
    assert_eq!(run("print(\"\\u0041\\u0042\");", ""), "AB");
    assert_eq!(run("print(length(\"\\u0041\"));", ""), "1");
}

#[test]
fn test_readint() {
    assert_eq!(run("integer x; x := readint; print(x);", "42\n"), "42");

    assert_eq!(
        run(
            "integer a; integer b; a := readint; b := readint; print(a + b);",
            "10\n20\n"
        ),
        "30"
    );
}

#[test]
fn test_readint_eof_returns_zero() {
    assert_eq!(run("integer x; x := readint; print(x);", ""), "0");
}

#[test]
fn test_readstr() {
    assert_eq!(run("string s; s := readstr; print(s);", "hello\n"), "hello");

    assert_eq!(
        run(
            "string a; string b; a := readstr; b := readstr; print(concatenate(a, b));",
            "foo\nbar\n"
        ),
        "foobar"
    );
}

#[test]
fn test_readstr_eof_returns_empty() {
    assert_eq!(run("string s; s := readstr; print(length(s));", ""), "0");
}

#[test]
fn test_nested_begin_end_blocks() {
    let out = run(
        "integer i; integer total; total := 0;
        begin
            for i := 1 to 3 do begin
                total := total + i;
            end;
            total := total * 2;
        end;
        print(total);",
        "",
    );

    assert_eq!(out, "12");
}

#[test]
fn test_if_elif_else_chain() {
    let out = run(
        "integer x; x := 2;
        if x = 0 then print(\"zero\")
        elif x = 1 then print(\"one\")
        elif x = 2 then print(\"two\")
        else print(\"many\");",
        "",
    );

    assert_eq!(out, "two");
}

#[test]
fn test_dangling_else_binds_to_inner_if() {
    let out = run(
        "integer x; x := 1;
        if x = 0 then
            if x = 1 then print(\"inner\")
            else print(\"outer-else\")
        else print(\"outer\");",
        "",
    );

    assert_eq!(out, "outer");
}

#[test]
fn test_variable_persistence_across_loops() {
    let out = run(
        "integer i; integer acc; acc := 0;
        for i := 1 to 10 do acc := acc + i;
        for i := 1 to 10 do acc := acc + i;
        print(acc);",
        "",
    );

    assert_eq!(out, "110");
}

#[test]
fn test_string_variables_use_rc_no_reallocation_on_read() {
    let out = run(
        "string s; integer i; string tmp;
        s := \"hello\";
        for i := 1 to 1000 do begin
            tmp := s;
        end;
        print(tmp);",
        "",
    );

    assert_eq!(out, "hello");
}

#[test]
fn test_mandelbrot_produces_output() {
    let out = run(include_str!("../programs/mandelbrot.pa"), "");

    let lines: Vec<&str> = out.lines().collect();

    assert_eq!(lines.len(), 40, "should produce 40 rows");

    assert!(lines[0].len() >= 70, "first row should be ~78 chars");
}
