use crate::tests::{check, check_error, compile};

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
        |main:
        |    push rbp
        |    mov rbp, rsp
        |    pop rbp
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
        |main:
        |    push rbp
        |    mov rbp, rsp
        |    mov eax, 0
        |    pop rbp
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
        |main:
        |    push rbp
        |    mov rbp, rsp
        |    mov eax, 1
        |    pop rbp
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
        |main:
        |    push rbp
        |    mov rbp, rsp
        |    mov eax, 42
        |    pop rbp
        |    ret
        |"#,
    );
}

#[test]
fn test_error_expected_declaration() {
    let output = compile(
        r#"
        |42
        |"#,
    );

    check_error(
        output,
        r#"
        |error: expected declaration, found `42`
        | >>> <source>:1:1
        |  |
        |1 | 42
        |  | ^^ expected declaration
        "#,
    );
}

#[test]
fn test_error_expected_colon_colon_after_identifier() {
    let output = compile(
        r#"
        |foo
        |"#,
    );

    check_error(
        output,
        r#"
        |error: expected `::`, found `<end of file>`
        | >>> <source>:1:1
        |  |
        |1 | foo
        |  | ^^^ expected `::` after this
        "#,
    );
}
