use crate::tests::{check, compile};

#[test]
fn test_call_previously_defined_function() {
    let program = compile(
        r#"
        |foo :: () {}
        |
        |bar :: () {
        |    foo()
        |}
        |"#,
    );

    check(
        program,
        r#"
        |foo:
        |    push rbp
        |    mov rbp, rsp
        |    pop rbp
        |    ret
        |
        |bar:
        |    push rbp
        |    mov rbp, rsp
        |    call foo
        |    pop rbp
        |    ret
        |"#,
    );
}

#[test]
fn test_call_later_defined_function() {
    let program = compile(
        r#"
        |foo :: () {
        |    bar()
        |}
        |
        |bar :: () {}
        |"#,
    );

    check(
        program,
        r#"
        |foo:
        |    push rbp
        |    mov rbp, rsp
        |    call bar
        |    pop rbp
        |    ret
        |
        |bar:
        |    push rbp
        |    mov rbp, rsp
        |    pop rbp
        |    ret
        |"#,
    );
}
