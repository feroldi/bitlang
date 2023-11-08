use std::fmt;

pub(crate) struct Diagnostic {
    pub(crate) compile_errors: Vec<CompileError>,
}

#[derive(Debug)]
pub(crate) enum CompileError {
    UnexpectedEndOfInput,
    ExpectedDeclaration,
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for compile_error in &self.compile_errors {
            writeln!(f, "error: {compile_error}, found `42`")?;
            writeln!(f, " >>> <source>:1:1")?;
            writeln!(f, "  |")?;
            writeln!(f, "1 | 42")?;
            writeln!(f, "  | ^^ {compile_error}")?;
        }

        Ok(())
    }
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompileError::ExpectedDeclaration => write!(f, "expected declaration"),
            ce => todo!("{:?}", ce),
        }
    }
}
