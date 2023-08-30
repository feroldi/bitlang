use crate::driver;

#[test]
fn test_main_empty_function_returns_0() {
    let program = compile(
        r#"
        |main :: () {}
        |"#,
    );

    check(
        program,
        r#"
        |.main:
        |    mov eax, 0
        |    ret
        |"#,
    );
}

#[test]
fn test_main_function_explicitly_returns_0() {
    let program = compile(
        r#"
        |main :: () -> i32 {
        |    0
        |}
        |"#,
    );

    check(
        program,
        r#"
        |.main:
        |    mov eax, 0
        |    ret
        |"#,
    );
}

#[test]
fn test_main_function_explicitly_returns_1() {
    let program = compile(
        r#"
        |main :: () -> i32 {
        |    1
        |}
        |"#,
    );

    check(
        program,
        r#"
        |.main:
        |    mov eax, 1
        |    ret
        |"#,
    );
}

#[test]
fn test_main_function_explicitly_returns_42() {
    let program = compile(
        r#"
        |main :: () -> i32 {
        |    42
        |}
        |"#,
    );

    check(
        program,
        r#"
        |.main:
        |    mov eax, 42
        |    ret
        |"#,
    );
}

#[test]
fn test_main_function_explicitly_returns_42() {
    let program = compile(
        r#"
        |42
        |"#,
    );

    check_fail(
        program,
        vec![Diag::ExpectedButGot {
            expected: TokenKind::Identifier,
            got: TokenKind::IntegerConstant,
        }],
    );
}

fn compile(source_code: &str) -> String {
    driver::compile(&strip_margin(source_code))
}

fn check<S: AsRef<str>>(program: S, expected_program: &str) {
    use pretty_assertions::assert_eq;

    assert_eq!(
        program.as_ref().trim(),
        strip_margin(expected_program).trim()
    );
}

fn strip_margin(text: &str) -> String {
    text.split('\n')
        .map(|line| {
            let mut stripped_line = line.chars().skip_while(|&c| c != '|');
            stripped_line.next();

            stripped_line.collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}
