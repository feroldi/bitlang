use crate::codegen::CodeGen;
use crate::parser::Parser;
use crate::scanner::Scanner;

pub(crate) fn compile(source_code: &str) -> String {
    let mut scanner = Scanner::new(source_code);
    let tokens = scanner.scan_all_tokens();

    let mut parser = Parser::new(source_code, tokens);
    let program = parser.parse_program().unwrap();

    let mut codegen = CodeGen::new();
    let x86_program = codegen.gen_program(program);

    format!("{}", x86_program)
}
