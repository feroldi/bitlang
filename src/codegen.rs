use crate::parser::{Const, Decl, Expr, Function, Program};
use std::fmt;

pub(crate) struct CodeGen {
    program: Program,
}

impl CodeGen {
    pub(crate) fn new(program: Program) -> CodeGen {
        CodeGen { program }
    }

    pub(crate) fn gen_program(&self) -> X86Program {
        let mut generated_insts = vec![];

        for decl in &self.program.decls {
            generated_insts.extend(self.gen_decl(decl));
        }

        X86Program {
            instructions: generated_insts,
        }
    }

    fn gen_decl(&self, decl: &Decl) -> Vec<Inst> {
        let mut decl_insts = vec![Inst::Label {
            name: decl.identifier.clone(),
        }];

        let value_insts = self.parse_top_level_expr(&decl.value);
        decl_insts.extend(value_insts);

        decl_insts
    }

    fn parse_top_level_expr(&self, expr: &Expr) -> Vec<Inst> {
        match expr {
            Expr::Function(Function { body, .. }) => self.gen_function(body),
            _ => todo!("other top-level exprs"),
        }
    }

    fn gen_function(&self, body: &[Expr]) -> Vec<Inst> {
        if body.is_empty() {
            vec![
                Inst::Mov {
                    target_reg: Reg::Eax,
                    value: 0,
                },
                Inst::Ret,
            ]
        } else {
            let mut body_insts = vec![];

            for expr in body {
                body_insts.extend(self.parse_expr(expr));
            }

            body_insts.push(Inst::Ret);

            body_insts
        }
    }

    fn parse_expr(&self, expr: &Expr) -> Vec<Inst> {
        match expr {
            Expr::Const(Const::IntegerConstant { value }) => {
                vec![Inst::Mov {
                    target_reg: Reg::Eax,
                    value: *value,
                }]
            }
            _ => todo!("other exprs"),
        }
    }
}

pub(crate) struct X86Program {
    instructions: Vec<Inst>,
}

enum Inst {
    Label { name: String },
    Mov { target_reg: Reg, value: i32 },
    Ret,
}

enum Reg {
    Eax,
}

impl fmt::Display for X86Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.instructions
            .iter()
            .try_fold((), |_, inst| writeln!(f, "{}", inst))
    }
}

impl fmt::Display for Inst {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Inst::Label { name } => write!(f, ".{}:", name),
            Inst::Mov { target_reg, value } => write!(f, "    mov {}, {}", target_reg, value),
            Inst::Ret => write!(f, "    ret"),
        }
    }
}

impl fmt::Display for Reg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Reg::Eax => write!(f, "eax"),
        }
    }
}
