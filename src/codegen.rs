use crate::parser::{Const, Decl, Expr, Function, Program};
use std::fmt;

pub(crate) struct CodeGen {
    label_counter: u64,
}

impl CodeGen {
    pub(crate) fn new() -> CodeGen {
        CodeGen { label_counter: 0 }
    }

    pub(crate) fn gen_program(&mut self, program: Program) -> X86Program {
        let mut generated_insts = vec![];

        for decl in &program.decls {
            generated_insts.extend(self.gen_decl(decl));
        }

        X86Program {
            instructions: generated_insts,
        }
    }

    fn gen_decl(&mut self, decl: &Decl) -> Vec<Inst> {
        let mut decl_insts = vec![Inst::Label {
            name: decl.identifier.clone(),
        }];

        let value_insts = self.parse_top_level_expr(&decl.value);
        decl_insts.extend(value_insts);

        decl_insts
    }

    fn parse_top_level_expr(&mut self, expr: &Expr) -> Vec<Inst> {
        match expr {
            Expr::Function(Function { body, .. }) => self.gen_function(body),
            _ => todo!("other top-level exprs"),
        }
    }

    fn gen_function(&mut self, body: &[Expr]) -> Vec<Inst> {
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
                body_insts.extend(self.gen_expr(expr));
            }

            body_insts.push(Inst::Ret);

            body_insts
        }
    }

    fn gen_expr(&mut self, expr: &Expr) -> Vec<Inst> {
        match expr {
            Expr::Semi(expr) => self.gen_expr(expr),
            Expr::Const(Const::IntegerConstant { value }) => {
                vec![Inst::Mov {
                    target_reg: Reg::Eax,
                    value: *value,
                }]
            }
            Expr::If(if_expr) => {
                let (first_branch_insts, mut next_label) =
                    self.gen_cond_and_branch(&if_expr.cond_expr, &if_expr.true_branch);

                let mut branches_insts = vec![];

                for branch in &if_expr.else_if_branches {
                    let mut branch_insts = vec![];

                    branch_insts.push(Inst::Label { name: next_label });

                    let (cond_insts, je_label) =
                        self.gen_cond_and_branch(&branch.cond_expr, &branch.true_branch);

                    branch_insts.extend(cond_insts);
                    branches_insts.push(branch_insts);

                    next_label = je_label;
                }

                let mut final_branch_insts = vec![];

                if let Some(ref final_branch) = if_expr.final_branch {
                    final_branch_insts.push(Inst::Label { name: next_label });
                    next_label = self.make_label();
                    final_branch_insts.extend(self.gen_exprs(final_branch));
                }

                let mut if_insts = vec![];
                let exit_label = next_label;

                if_insts.extend(first_branch_insts);

                if !branches_insts.is_empty() || !final_branch_insts.is_empty() {
                    if_insts.push(Inst::Jmp {
                        label: exit_label.clone(),
                    });
                }

                for branch_insts in branches_insts {
                    if_insts.extend(branch_insts);
                    if_insts.push(Inst::Jmp {
                        label: exit_label.clone(),
                    });
                }

                if !final_branch_insts.is_empty() {
                    if_insts.extend(final_branch_insts);
                }

                if_insts.push(Inst::Label { name: exit_label });

                if_insts
            }
            _ => todo!("other exprs"),
        }
    }

    fn gen_cond_and_branch(&mut self, cond_expr: &Expr, branch: &[Expr]) -> (Vec<Inst>, String) {
        let mut insts = self.gen_expr(cond_expr);

        insts.push(Inst::Cmp {
            reg: Reg::Eax,
            value: 0,
        });

        let next_branch_label = self.make_label();
        insts.push(Inst::Je {
            label: next_branch_label.clone(),
        });

        insts.extend(self.gen_exprs(branch));

        (insts, next_branch_label)
    }

    fn gen_exprs(&mut self, exprs: &[Expr]) -> Vec<Inst> {
        exprs.iter().flat_map(|e| self.gen_expr(e)).collect()
    }

    fn make_label(&mut self) -> String {
        let label_count = self.label_counter;
        self.label_counter += 1;

        format!("L{}", label_count)
    }
}

pub(crate) struct X86Program {
    instructions: Vec<Inst>,
}

enum Inst {
    Label { name: String },
    Mov { target_reg: Reg, value: i32 },
    Cmp { reg: Reg, value: i32 },
    Je { label: String },
    Jmp { label: String },
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
            Inst::Cmp { reg, value } => write!(f, "    cmp {}, {}", reg, value),
            Inst::Je { label } => write!(f, "    je .{}", label),
            Inst::Jmp { label } => write!(f, "    jmp .{}", label),
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
