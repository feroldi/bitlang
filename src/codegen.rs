use std::collections::HashMap;
use std::fmt;

use crate::ast::{BindDef, BindRef, Const, Decl, Expr, Function, IfExpr, Program};
use crate::compiler_context::CompilerContext;
use crate::interner::Symbol;

pub(crate) struct CodeGen<'ctx> {
    ctx: &'ctx CompilerContext,
    label_counter: u64,
    allocated_stack_bytes: u64,
    offset_by_bind_ref: HashMap<Symbol, u64>,
}

impl<'ctx> CodeGen<'ctx> {
    pub(crate) fn new(ctx: &'ctx CompilerContext) -> CodeGen<'ctx> {
        CodeGen {
            ctx,
            label_counter: 0,
            allocated_stack_bytes: 0,
            offset_by_bind_ref: Default::default(),
        }
    }

    pub(crate) fn gen_program(&mut self, program: Program) -> X86Program {
        let mut generated_insts = vec![];

        for decl in program.decls {
            generated_insts.extend(self.gen_decl(decl));
        }

        X86Program {
            ctx: self.ctx,
            instructions: generated_insts,
        }
    }

    fn gen_decl(&mut self, decl: &Decl) -> Vec<Inst> {
        let mut decl_insts = vec![Inst::Label {
            name: decl.identifier,
        }];

        let value_insts = self.parse_top_level_expr(decl.value);
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
        let mut insts = vec![
            Inst::Push { source: Reg::Rbp },
            Inst::Mov {
                target: Arg::Reg(Reg::Rbp),
                source: Arg::Reg(Reg::Rsp),
            },
        ];

        let mut body_insts = vec![];
        for expr in body {
            body_insts.extend(self.gen_expr(expr));
        }

        if self.allocated_stack_bytes != 0 {
            // FIXME: Should not cast allocated_stack_bytes to i32.
            insts.push(Inst::Sub {
                target: Arg::Reg(Reg::Rsp),
                source: Arg::Imm(self.allocated_stack_bytes as i32),
            });

            // FIXME: Should not cast allocated_stack_bytes to i32.
            body_insts.push(Inst::Add {
                target: Arg::Reg(Reg::Rsp),
                source: Arg::Imm(self.allocated_stack_bytes as i32),
            });
        }

        body_insts.push(Inst::Pop { target: Reg::Rbp });
        body_insts.push(Inst::Ret);

        insts.extend(body_insts);

        // FIXME: This is obviously temporary. We will need stack frames, because we
        // have inner scopes, and those cannot simply clear the current stack
        // frame.
        self.allocated_stack_bytes = 0;
        self.offset_by_bind_ref.clear();

        insts
    }

    fn gen_expr(&mut self, expr: &Expr) -> Vec<Inst> {
        match expr {
            Expr::Semi(expr) => self.gen_expr(expr),
            Expr::Const(constant) => self.gen_constant_expr(*constant),
            Expr::If(if_expr) => self.gen_if_expr(*if_expr),
            Expr::BindDef(bind_def) => self.gen_bind_def_expr(*bind_def),
            Expr::BindRef(bind_ref) => self.gen_bind_ref_expr(*bind_ref),
            Expr::Function(_) => unimplemented!()
        }
    }

    fn gen_constant_expr(&self, constant: Const) -> Vec<Inst> {
        match constant {
            Const::IntegerConstant { value } => {
                vec![Inst::Mov {
                    target: Arg::Reg(Reg::Eax),
                    source: Arg::Imm(value),
                }]
            }
        }
    }

    fn gen_if_expr(&mut self, if_expr: IfExpr) -> Vec<Inst> {
        let (first_branch_insts, mut next_label) =
            self.gen_cond_and_branch(if_expr.cond_expr, if_expr.true_branch);

        let mut branches_insts = vec![];

        for branch in if_expr.else_if_branches {
            let mut branch_insts = vec![];

            branch_insts.push(Inst::Label { name: next_label });

            let (cond_insts, je_label) =
                self.gen_cond_and_branch(branch.cond_expr, branch.true_branch);

            branch_insts.extend(cond_insts);
            branches_insts.push(branch_insts);

            next_label = je_label;
        }

        let mut final_branch_insts = vec![];

        if let Some(final_branch) = if_expr.final_branch {
            final_branch_insts.push(Inst::Label { name: next_label });
            next_label = self.make_label();
            final_branch_insts.extend(self.gen_exprs(final_branch));
        }

        let mut if_insts = vec![];
        let exit_label = next_label;

        if_insts.extend(first_branch_insts);

        if !branches_insts.is_empty() || !final_branch_insts.is_empty() {
            if_insts.push(Inst::Jmp { label: exit_label });
        }

        for branch_insts in branches_insts {
            if_insts.extend(branch_insts);
            if_insts.push(Inst::Jmp { label: exit_label });
        }

        if !final_branch_insts.is_empty() {
            if_insts.extend(final_branch_insts);
        }

        if_insts.push(Inst::Label { name: exit_label });

        if_insts
    }

    fn gen_cond_and_branch(&mut self, cond_expr: &Expr, branch: &[Expr]) -> (Vec<Inst>, Symbol) {
        let mut insts = self.gen_expr(cond_expr);

        insts.push(Inst::Cmp {
            reg: Reg::Eax,
            value: 0,
        });

        let next_branch_label = self.make_label();
        insts.push(Inst::Je {
            label: next_branch_label,
        });

        insts.extend(self.gen_exprs(branch));

        (insts, next_branch_label)
    }

    fn gen_bind_def_expr(&mut self, bind_def: BindDef) -> Vec<Inst> {
        let mut insts = self.gen_expr(bind_def.value);

        self.allocated_stack_bytes += 4;

        self.offset_by_bind_ref
            .insert(bind_def.identifier, self.allocated_stack_bytes);

        insts.push(Inst::Mov {
            target: Arg::MemOffset {
                base: Reg::Rbp,
                // FIXME: Should not cast allocated_stack_bytes to i32.
                offset: -(self.allocated_stack_bytes as i32),
            },
            source: Arg::Reg(Reg::Eax),
        });

        insts
    }

    fn gen_bind_ref_expr(&mut self, bind_ref: BindRef) -> Vec<Inst> {
        let bind_offset = self.offset_by_bind_ref.get(&bind_ref.identifier).unwrap();

        vec![Inst::Mov {
            target: Arg::Reg(Reg::Eax),
            source: Arg::MemOffset {
                base: Reg::Rbp,
                offset: -(*bind_offset as i32),
            },
        }]
    }

    fn gen_exprs(&mut self, exprs: &[Expr]) -> Vec<Inst> {
        exprs.iter().flat_map(|e| self.gen_expr(e)).collect()
    }

    fn make_label(&mut self) -> Symbol {
        let label_count = self.label_counter;
        self.label_counter += 1;

        self.ctx.get_or_intern_str(&format!("L{}", label_count))
    }
}

pub(crate) struct X86Program<'ctx> {
    ctx: &'ctx CompilerContext,
    instructions: Vec<Inst>,
}

#[derive(Clone, Copy)]
enum Inst {
    Label { name: Symbol },
    Mov { target: Arg, source: Arg },
    Cmp { reg: Reg, value: i32 },
    Je { label: Symbol },
    Jmp { label: Symbol },
    Ret,
    Push { source: Reg },
    Pop { target: Reg },
    Sub { target: Arg, source: Arg },
    Add { target: Arg, source: Arg },
}

#[derive(Clone, Copy)]
enum Arg {
    Imm(i32),
    Reg(Reg),
    MemOffset { base: Reg, offset: i32 },
}

#[derive(Clone, Copy)]
enum Reg {
    Eax,
    Rbp,
    Rsp,
}

impl fmt::Display for X86Program<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for inst in &self.instructions {
            let ctx_inst = CtxInst {
                ctx: self.ctx,
                inst: *inst,
            };
            writeln!(f, "{}", ctx_inst)?;
        }

        Ok(())
    }
}

struct CtxInst<'ctx> {
    ctx: &'ctx CompilerContext,
    inst: Inst,
}

impl fmt::Display for CtxInst<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !matches!(self.inst, Inst::Label { .. }) {
            write!(f, "    ")?;
        }

        match self.inst {
            Inst::Label { name } => write!(f, ".{}:", self.ctx.resolve_symbol(name)),
            Inst::Mov { target, source } => write!(f, "mov {}, {}", target, source),
            Inst::Cmp { reg, value } => write!(f, "cmp {}, {}", reg, value),
            Inst::Je { label } => write!(f, "je .{}", self.ctx.resolve_symbol(label)),
            Inst::Jmp { label } => write!(f, "jmp .{}", self.ctx.resolve_symbol(label)),
            Inst::Ret => write!(f, "ret"),
            Inst::Push { source } => write!(f, "push {}", source),
            Inst::Pop { target } => write!(f, "pop {}", target),
            Inst::Sub { target, source } => write!(f, "sub {}, {}", target, source),
            Inst::Add { target, source } => write!(f, "add {}, {}", target, source),
        }
    }
}

impl fmt::Display for Arg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Arg::Imm(value) => write!(f, "{}", value),
            Arg::Reg(reg) => write!(f, "{}", reg),
            Arg::MemOffset { base, offset } => {
                write!(
                    f,
                    "DWORD PTR [{base}{sign}{offset}]",
                    base = base,
                    sign = if *offset >= 0 { "+" } else { "" },
                    offset = offset
                )
            }
        }
    }
}

impl fmt::Display for Reg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Reg::Eax => write!(f, "eax"),
            Reg::Rbp => write!(f, "rbp"),
            Reg::Rsp => write!(f, "rsp"),
        }
    }
}
