use crate::codegen::CodeGen;
use crate::compiler_context::CompilerContext;
use crate::parser::Parser;
use crate::scanner::Scanner;

pub(crate) fn compile(source_code: &str) -> String {
    // FIXME: don't copy source code, move it.
    let context = CompilerContext::new(source_code.into());

    let mut scanner = Scanner::new(&context);
    let tokens = scanner.scan_all_tokens();

    let mut parser = Parser::new(tokens, &context);
    let program = parser.parse_program().unwrap();

    let mut codegen = CodeGen::new(&context);
    let x86_program = codegen.gen_program(program);

    format!("{}", x86_program)
}
