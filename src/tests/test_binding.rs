use crate::tests::{check, compile};

#[test]
fn test_bind_to_primary_expr_and_return_from_function() {
    let program = compile(
        r#"
        |main :: () -> i32 {
        |    foo := 42;
        |    foo
        |}
        |"#,
    );

    check(
        program,
        r#"
        |.main:
        |    push rbp
        |    mov rbp, rsp
        |    sub rsp, 4
        |
        |    mov eax, 42
        |    mov DWORD PTR [rbp-4], eax
        |
        |    mov eax, DWORD PTR [rbp-4]
        |
        |    pop rbp
        |    ret
        |"#,
    );
}

#[test]
fn test_allocate_stack_according_to_how_many_bindings_there_are_and_ref_then_back() {
    let program = compile(
        r#"
        |main :: () -> i32 {
        |    foo := 42;
        |    bar := 314;
        |    baz := 1;
        |    quxx := 0;
        |
        |    bar
        |}
        |"#,
    );

    check(
        program,
        r#"
        |.main:
        |    push rbp
        |    mov rbp, rsp
        |    sub rsp, 16
        |
        |    mov eax, 42
        |    mov DWORD PTR [rbp-4], eax
        |
        |    mov eax, 314
        |    mov DWORD PTR [rbp-8], eax
        |
        |    mov eax, 1
        |    mov DWORD PTR [rbp-12], eax
        |
        |    mov eax, 0
        |    mov DWORD PTR [rbp-16], eax
        |
        |    mov eax, DWORD PTR [rbp-8]
        |
        |    pop rbp
        |    ret
        |"#,
    );
}
