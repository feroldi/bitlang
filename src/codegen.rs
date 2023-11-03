use std::collections::HashMap;
use std::fmt;

use crate::ast::{
    BindDef, BindRef, CompoundExpr, Const, Decl, Expr, FnCallExpr, ForExpr, ForIteration, Function,
    IfExpr, Program, RangeKind,
};
use crate::compiler_context::CompilerContext;
use crate::interner::Symbol;

pub(crate) struct CodeGen<'ctx> {
    ctx: &'ctx CompilerContext,
    label_counter: u64,
    allocated_stack_bytes: usize,
    scope_stack: Vec<Scope>,
}

#[derive(Default)]
pub(crate) struct Scope {
    memory_offset_by_symbol: HashMap<Symbol, usize>,
    innermost_exit_label: Option<Symbol>,
}

impl<'ctx> CodeGen<'ctx> {
    pub(crate) fn new(ctx: &'ctx CompilerContext) -> CodeGen<'ctx> {
        CodeGen {
            ctx,
            label_counter: 0,
            allocated_stack_bytes: 0,
            scope_stack: vec![],
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
            Expr::Function(Function { body, .. }) => self.gen_function(*body),
            _ => todo!("other top-level exprs"),
        }
    }

    fn gen_function(&mut self, body: CompoundExpr) -> Vec<Inst> {
        self.enter_scope();

        let mut insts = vec![
            Inst::Push { source: Reg::Rbp },
            Inst::Mov {
                target: Arg::Reg(Reg::Rbp),
                source: Arg::Reg(Reg::Rsp),
            },
        ];

        let mut body_insts = self.gen_compound_expr(body);

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

        self.exit_scope();

        insts
    }

    fn gen_expr(&mut self, expr: &Expr) -> Vec<Inst> {
        match expr {
            Expr::Semi(expr) => self.gen_expr(expr),
            Expr::Const(constant) => self.gen_constant_expr(*constant),
            Expr::If(if_expr) => self.gen_if_expr(*if_expr),
            Expr::For(for_expr) => self.gen_for_expr(*for_expr),
            Expr::Break => self.gen_break_expr(),
            Expr::BindDef(bind_def) => self.gen_bind_def_expr(*bind_def),
            Expr::BindRef(bind_ref) => self.gen_bind_ref_expr(*bind_ref),
            Expr::Compound(compound_expr) => self.gen_compound_expr(*compound_expr),
            Expr::FnCall(fn_call_expr) => self.gen_fn_call_expr(*fn_call_expr),
            Expr::Function(_) => unimplemented!(),
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
            final_branch_insts.extend(self.gen_compound_expr(final_branch));
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

    fn gen_cond_and_branch(
        &mut self,
        cond_expr: &Expr,
        branch: CompoundExpr,
    ) -> (Vec<Inst>, Symbol) {
        let mut insts = self.gen_expr(cond_expr);

        insts.push(Inst::Cmp {
            reg: Reg::Eax,
            value: 0,
        });

        let next_branch_label = self.make_label();
        insts.push(Inst::Je {
            label: next_branch_label,
        });

        insts.extend(self.gen_compound_expr(branch));

        (insts, next_branch_label)
    }

    fn gen_for_expr(&mut self, for_expr: ForExpr) -> Vec<Inst> {
        let start_label = self.make_label();
        let exit_label = self.make_label();

        self.set_innermost_exit_label(exit_label);

        let mut insts = vec![];

        self.enter_scope();

        match for_expr.iteration {
            Some(ForIteration::Conditional { cond_expr }) => {
                insts.push(Inst::Label { name: start_label });

                insts.extend(self.gen_expr(cond_expr));
                insts.push(Inst::Cmp {
                    reg: Reg::Eax,
                    value: 0,
                });

                insts.push(Inst::Je { label: exit_label });
                insts.extend(self.gen_compound_expr(for_expr.body));
            }
            Some(ForIteration::Iterative {
                identifier,
                start_expr,
                end_expr,
                range_kind,
            }) => {
                insts.extend(self.gen_bind_def_expr(BindDef {
                    identifier,
                    value: start_expr,
                }));

                insts.push(Inst::Label { name: start_label });

                let bind_ref = BindRef { identifier };

                insts.extend(self.gen_bind_ref_expr(bind_ref));
                // FIXME: This is specialized because I can't allocate registers at will.
                let value = match end_expr {
                    Expr::Const(Const::IntegerConstant { value }) => *value,
                    _ => unimplemented!(),
                };

                insts.push(Inst::Cmp {
                    reg: Reg::Eax,
                    value,
                });
                match range_kind {
                    RangeKind::Inclusive => insts.push(Inst::Jg { label: exit_label }),
                    RangeKind::Exclusive => insts.push(Inst::Jge { label: exit_label }),
                }

                insts.extend(self.gen_compound_expr(for_expr.body));

                insts.extend(self.gen_bind_ref_expr(bind_ref));
                insts.push(Inst::Add {
                    target: Arg::Reg(Reg::Eax),
                    source: Arg::Imm(1),
                });
                let bind_offset = self.get_in_scope(bind_ref);
                insts.push(Inst::Mov {
                    target: Arg::MemOffset {
                        base: Reg::Rbp,
                        offset: -(bind_offset as i32),
                    },
                    source: Arg::Reg(Reg::Eax),
                });
            }
            None => {
                insts.push(Inst::Label { name: start_label });
                insts.extend(self.gen_compound_expr(for_expr.body));
            }
        }

        insts.push(Inst::Jmp { label: start_label });
        insts.push(Inst::Label { name: exit_label });

        self.exit_scope();

        insts
    }

    fn gen_break_expr(&mut self) -> Vec<Inst> {
        let exit_label = self.get_innermost_exit_label();

        vec![Inst::Jmp { label: exit_label }]
    }

    fn gen_bind_def_expr(&mut self, bind_def: BindDef) -> Vec<Inst> {
        let mut insts = self.gen_expr(bind_def.value);

        let offset = self.insert_in_scope(bind_def);

        insts.push(Inst::Mov {
            target: Arg::MemOffset {
                base: Reg::Rbp,
                // FIXME: Should not cast allocated_stack_bytes to i32.
                offset: -(offset as i32),
            },
            source: Arg::Reg(Reg::Eax),
        });

        insts
    }

    fn gen_bind_ref_expr(&mut self, bind_ref: BindRef) -> Vec<Inst> {
        let bind_offset = self.get_in_scope(bind_ref);

        vec![Inst::Mov {
            target: Arg::Reg(Reg::Eax),
            source: Arg::MemOffset {
                base: Reg::Rbp,
                offset: -(bind_offset as i32),
            },
        }]
    }

    fn gen_compound_expr(&mut self, compound_expr: CompoundExpr) -> Vec<Inst> {
        self.enter_scope();
        let insts = compound_expr
            .exprs
            .iter()
            .flat_map(|e| self.gen_expr(e))
            .collect();
        self.exit_scope();

        insts
    }

    fn gen_fn_call_expr(&mut self, fn_call_expr: FnCallExpr) -> Vec<Inst> {
        vec![Inst::Call {
            label: fn_call_expr.identifier,
        }]
    }

    fn make_label(&mut self) -> Symbol {
        let label_count = self.label_counter;
        self.label_counter += 1;

        self.ctx.get_or_intern_str(&format!(".L{}", label_count))
    }

    fn enter_scope(&mut self) {
        self.scope_stack.push(Scope::default());
    }

    fn exit_scope(&mut self) {
        self.scope_stack.pop();

        if self.scope_stack.is_empty() {
            self.allocated_stack_bytes = 0;
        }
    }

    fn get_this_scope_mut(&mut self) -> &mut Scope {
        self.scope_stack.last_mut().unwrap()
    }

    fn insert_in_scope(&mut self, bind_def: BindDef) -> usize {
        let bumped_allocated_stack_bytes = self.allocated_stack_bytes + 4;

        self.get_this_scope_mut()
            .memory_offset_by_symbol
            .insert(bind_def.identifier, bumped_allocated_stack_bytes);

        self.allocated_stack_bytes = bumped_allocated_stack_bytes;

        self.allocated_stack_bytes
    }

    fn get_in_scope(&self, bind_ref: BindRef) -> usize {
        for scope in self.scope_stack.iter().rev() {
            if let Some(bind_offset) = scope.memory_offset_by_symbol.get(&bind_ref.identifier) {
                return *bind_offset;
            }
        }

        unreachable!("binding does not exist")
    }

    fn find_in_scope<R: Clone, F: Fn(&Scope) -> Option<R>>(&self, f: F) -> R {
        for scope in self.scope_stack.iter().rev() {
            if let Some(r) = f(scope) {
                return r;
            }
        }

        unreachable!("scope does not exist")
    }

    fn get_innermost_exit_label(&self) -> Symbol {
        self.find_in_scope(|scope| scope.innermost_exit_label)
    }

    fn set_innermost_exit_label(&mut self, exit_label: Symbol) {
        self.get_this_scope_mut().innermost_exit_label = Some(exit_label)
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
    Jg { label: Symbol },
    Jge { label: Symbol },
    Jmp { label: Symbol },
    Ret,
    Push { source: Reg },
    Pop { target: Reg },
    Sub { target: Arg, source: Arg },
    Add { target: Arg, source: Arg },
    Call { label: Symbol },
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
            Inst::Label { name } => write!(f, "{}:", self.ctx.resolve_symbol(name)),
            Inst::Mov { target, source } => write!(f, "mov {}, {}", target, source),
            Inst::Cmp { reg, value } => write!(f, "cmp {}, {}", reg, value),
            Inst::Je { label } => write!(f, "je {}", self.ctx.resolve_symbol(label)),
            Inst::Jg { label } => write!(f, "jg {}", self.ctx.resolve_symbol(label)),
            Inst::Jge { label } => write!(f, "jge {}", self.ctx.resolve_symbol(label)),
            Inst::Jmp { label } => write!(f, "jmp {}", self.ctx.resolve_symbol(label)),
            Inst::Ret => write!(f, "ret"),
            Inst::Push { source } => write!(f, "push {}", source),
            Inst::Pop { target } => write!(f, "pop {}", target),
            Inst::Sub { target, source } => write!(f, "sub {}, {}", target, source),
            Inst::Add { target, source } => write!(f, "add {}, {}", target, source),
            Inst::Call { label } => write!(f, "call {}", self.ctx.resolve_symbol(label)),
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
