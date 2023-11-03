use crate::interner::Symbol;

#[derive(Clone, Copy)]
pub(crate) struct Program<'ctx> {
    pub(crate) decls: &'ctx [Decl<'ctx>],
}

#[derive(Clone, Copy)]
pub(crate) struct Decl<'ctx> {
    pub(crate) identifier: Symbol,
    pub(crate) value: &'ctx Expr<'ctx>,
}

#[derive(Clone, Copy)]
pub(crate) enum Expr<'ctx> {
    Const(Const),
    BindRef(BindRef),
    BindDef(BindDef<'ctx>),
    Function(Function<'ctx>),
    If(IfExpr<'ctx>),
    For(ForExpr<'ctx>),
    Break,
    Compound(CompoundExpr<'ctx>),
    Semi(&'ctx Expr<'ctx>),
    FnCall(FnCallExpr),
}

#[derive(Clone, Copy)]
pub(crate) enum Const {
    IntegerConstant { value: i32 },
}

#[derive(Clone, Copy)]
pub(crate) struct BindRef {
    pub(crate) identifier: Symbol,
}

#[derive(Clone, Copy)]
pub(crate) struct BindDef<'ctx> {
    pub(crate) identifier: Symbol,
    pub(crate) value: &'ctx Expr<'ctx>,
}

#[derive(Clone, Copy)]
pub(crate) struct Function<'ctx> {
    pub(crate) return_type: Type,
    pub(crate) parameters: &'ctx [Param],
    pub(crate) body: CompoundExpr<'ctx>,
}

#[derive(Clone, Copy)]
pub(crate) struct Param {
    identifier: Symbol,
    ty: Type,
}

#[derive(Clone, Copy)]
pub(crate) enum Type {
    Unit,
    I32,
}

#[derive(Clone, Copy)]
pub(crate) struct IfExpr<'ctx> {
    pub(crate) cond_expr: &'ctx Expr<'ctx>,
    pub(crate) true_branch: CompoundExpr<'ctx>,
    pub(crate) else_if_branches: &'ctx [ElseIfBranch<'ctx>],
    pub(crate) final_branch: Option<CompoundExpr<'ctx>>,
}

#[derive(Clone, Copy)]
pub(crate) struct ElseIfBranch<'ctx> {
    pub(crate) cond_expr: &'ctx Expr<'ctx>,
    pub(crate) true_branch: CompoundExpr<'ctx>,
}

#[derive(Clone, Copy)]
pub(crate) struct ForExpr<'ctx> {
    pub(crate) iteration: Option<ForIteration<'ctx>>,
    pub(crate) body: CompoundExpr<'ctx>,
}

#[derive(Clone, Copy)]
pub(crate) enum ForIteration<'ctx> {
    Conditional {
        cond_expr: &'ctx Expr<'ctx>,
    },
    Iterative {
        identifier: Symbol,
        start_expr: &'ctx Expr<'ctx>,
        end_expr: &'ctx Expr<'ctx>,
        range_kind: RangeKind,
    },
}

#[derive(Clone, Copy)]
pub(crate) enum RangeKind {
    Inclusive,
    Exclusive,
}

#[derive(Clone, Copy)]
pub(crate) struct CompoundExpr<'ctx> {
    pub(crate) exprs: &'ctx [Expr<'ctx>],
}

#[derive(Clone, Copy)]
pub(crate) struct FnCallExpr {
    pub(crate) identifier: Symbol,
}
