use crate::tests::{check, compile};

#[test]
fn test_if_else_with_basic_expressions() {
    let program = compile(
        r#"
        |main :: () -> i32 {
        |    if 1 {
        |        1
        |    } else {
        |        0
        |    }
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
        |    cmp eax, 0
        |    je .L0
        |    mov eax, 1
        |    jmp .L1
        |
        |.L0:
        |    mov eax, 0
        |
        |.L1:
        |    pop rbp
        |    ret
        |"#,
    );
}

#[test]
fn test_if_else_inside_another() {
    let program = compile(
        r#"
        |main :: () -> i32 {
        |    if 1 {
        |        if 2 { 3 } else { 4 }
        |    } else {
        |        0
        |    }
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
        |    cmp eax, 0
        |    je .L0
        |    mov eax, 2
        |    cmp eax, 0
        |    je .L1
        |    mov eax, 3
        |    jmp .L2
        |.L1:
        |    mov eax, 4
        |.L2:
        |    jmp .L3
        |.L0:
        |    mov eax, 0
        |.L3:
        |    pop rbp
        |    ret
        |"#,
    );
}

#[test]
fn test_if_without_else() {
    let program = compile(
        r#"
        |main :: () -> i32 {
        |    if 1 {
        |        1
        |    };
        |
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
        |    mov eax, 1
        |    cmp eax, 0
        |    je .L0
        |    mov eax, 1
        |.L0:
        |    mov eax, 0
        |    pop rbp
        |    ret
        |"#,
    );
}

#[test]
fn test_multi_exprs_inside_if_body() {
    let program = compile(
        r#"
        |main :: () -> i32 {
        |    if 1 {
        |        1;
        |        2;
        |        3
        |    } else {
        |        4;
        |        5
        |    }
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
        |    cmp eax, 0
        |    je .L0
        |    mov eax, 1
        |    mov eax, 2
        |    mov eax, 3
        |    jmp .L1
        |.L0:
        |    mov eax, 4
        |    mov eax, 5
        |.L1:
        |    pop rbp
        |    ret
        |"#,
    );
}

#[test]
fn test_chained_if_else() {
    let program = compile(
        r#"
        |main :: () -> i32 {
        |    if 10 {
        |        1
        |    } else if 20 {
        |        2
        |    } else if 30 {
        |        3
        |    } else {
        |        4
        |    }
        |}
        |"#,
    );

    check(
        program,
        r#"
        |main:
        |    push rbp
        |    mov rbp, rsp
        |    mov eax, 10
        |    cmp eax, 0
        |    je .L0
        |    mov eax, 1
        |    jmp .L3
        |
        |.L0:
        |    mov eax, 20
        |    cmp eax, 0
        |    je .L1
        |    mov eax, 2
        |    jmp .L3
        |
        |.L1:
        |    mov eax, 30
        |    cmp eax, 0
        |    je .L2
        |    mov eax, 3
        |    jmp .L3
        |
        |.L2:
        |    mov eax, 4
        |.L3:
        |    pop rbp
        |    ret
        |"#,
    );
}

#[test]
fn test_chained_if_else_without_final_else() {
    let program = compile(
        r#"
        |main :: () -> i32 {
        |    if 10 {
        |        1
        |    } else if 20 {
        |        2
        |    } else if 30 {
        |        3
        |    }
        |}
        |"#,
    );

    check(
        program,
        r#"
        |main:
        |    push rbp
        |    mov rbp, rsp
        |    mov eax, 10
        |    cmp eax, 0
        |    je .L0
        |    mov eax, 1
        |    jmp .L2
        |
        |.L0:
        |    mov eax, 20
        |    cmp eax, 0
        |    je .L1
        |    mov eax, 2
        |    jmp .L2
        |
        |.L1:
        |    mov eax, 30
        |    cmp eax, 0
        |    je .L2
        |    mov eax, 3
        |    jmp .L2
        |
        |.L2:
        |    pop rbp
        |    ret
        |"#,
    );
}
