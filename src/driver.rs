use crate::codegen::CodeGen;
use crate::compiler_context::CompilerContext;
use crate::parser::Parser;
use crate::scanner::Scanner;

pub(crate) fn compile(source_code: &str) -> String {
    // FIXME: don't copy source code, move it.
    let context = CompilerContext::new(source_code.into());

    let tokens = {
        let mut scanner = Scanner::new(&context);
        scanner.scan_all_tokens()
    };

    let mut parser = Parser::new(tokens, &context);

    match parser.parse_program() {
        Ok(program) => {
            let mut codegen = CodeGen::new(&context);
            let x86_program = codegen.gen_program(program);

            format!("{}", x86_program)
        }
        Err(diagnostic) => {
            format!("{}", diagnostic)
        }
    }
}
