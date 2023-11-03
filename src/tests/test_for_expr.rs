use crate::tests::{check, compile};

#[test]
fn test_empty_infinite_for_loop() {
    let program = compile(
        r#"
        |main :: () {
        |    for {
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
        |.L0:
        |    jmp .L0
        |.L1:
        |    pop rbp
        |    ret
        |"#,
    );
}

#[test]
fn test_infinite_for_loop() {
    let program = compile(
        r#"
        |main :: () {
        |    for {
        |        x := 42;
        |        foo();
        |    }
        |}
        |
        |foo :: () {}
        |"#,
    );

    check(
        program,
        r#"
        |main:
        |    push rbp
        |    mov rbp, rsp
        |    sub rsp, 4
        |
        |.L0:
        |    mov eax, 42
        |    mov DWORD PTR [rbp-4], eax
        |    call foo
        |    jmp .L0
        |
        |.L1:
        |    add rsp, 4
        |    pop rbp
        |    ret
        |
        |foo:
        |    push rbp
        |    mov rbp, rsp
        |    pop rbp
        |    ret
        |"#,
    );
}

#[test]
fn test_conditional_empty_infinite_for_loop() {
    let program = compile(
        r#"
        |main :: () {
        |    for 1 {
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
        |.L0:
        |    mov eax, 1
        |    cmp eax, 0
        |    je .L1
        |    jmp .L0
        |.L1:
        |    pop rbp
        |    ret
        |"#,
    );
}

#[test]
fn test_conditional_for_loop() {
    let program = compile(
        r#"
        |main :: () {
        |    for 1 {
        |        foo();
        |    }
        |}
        |
        |foo :: () {}
        |"#,
    );

    check(
        program,
        r#"
        |main:
        |    push rbp
        |    mov rbp, rsp
        |
        |.L0:
        |    mov eax, 1
        |    cmp eax, 0
        |    je .L1
        |
        |    call foo
        |
        |    jmp .L0
        |
        |.L1:
        |    pop rbp
        |    ret
        |
        |foo:
        |    push rbp
        |    mov rbp, rsp
        |    pop rbp
        |    ret
        |"#,
    );
}

#[test]
fn test_iteration_for_loop_exclusive_range() {
    let program = compile(
        r#"
        |main :: () {
        |    for i : 3..10 {
        |        i;
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
        |    sub rsp, 4
        |
        |    mov eax, 3
        |    mov DWORD PTR [rbp-4], eax
        |
        |.L0:
        |    mov eax, DWORD PTR [rbp-4]
        |    cmp eax, 10
        |    jge .L1
        |
        |    mov eax, DWORD PTR [rbp-4]
        |
        |    mov eax, DWORD PTR [rbp-4]
        |    add eax, 1
        |    mov DWORD PTR [rbp-4], eax
        |    jmp .L0
        |
        |.L1:
        |    add rsp, 4
        |    pop rbp
        |    ret
        |"#,
    );
}

#[test]
fn test_iteration_for_loop_inclusive_range() {
    let program = compile(
        r#"
        |main :: () {
        |    for i : 3..=10 {
        |        i;
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
        |    sub rsp, 4
        |
        |    mov eax, 3
        |    mov DWORD PTR [rbp-4], eax
        |
        |.L0:
        |    mov eax, DWORD PTR [rbp-4]
        |    cmp eax, 10
        |    jg .L1
        |
        |    mov eax, DWORD PTR [rbp-4]
        |
        |    mov eax, DWORD PTR [rbp-4]
        |    add eax, 1
        |    mov DWORD PTR [rbp-4], eax
        |    jmp .L0
        |
        |.L1:
        |    add rsp, 4
        |    pop rbp
        |    ret
        |"#,
    );
}


#[test]
fn test_break_infinite_for_loop() {
    let program = compile(
        r#"
        |main :: () {
        |    for {
        |        break
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
        |.L0:
        |    jmp .L1
        |    jmp .L0
        |.L1:
        |    pop rbp
        |    ret
        |"#,
    );
}

#[test]
fn test_break_innermost_for_loop() {
    let program = compile(
        r#"
        |main :: () {
        |    for {
        |        for {
        |            break
        |        }
        |        break
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
        |.L0:               ; start of outermost for-loop
        |
        |.L2:               ; start of innermost for-loop
        |    jmp .L3        ; break out of innermost for-loop
        |    jmp .L2
        |.L3:               ; exit of innermost for-loop
        |
        |    jmp .L1        ; break out of outermost for-loop
        |    jmp .L0
        |
        |.L1:               ; exit of outermost for-loop
        |    pop rbp
        |    ret
        |"#,
    );
}
