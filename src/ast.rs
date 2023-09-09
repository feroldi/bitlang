#[derive(Clone, Copy)]
pub(crate) struct Program<'ctx> {
    pub(crate) decls: &'ctx [Decl<'ctx>],
}

#[derive(Clone, Copy)]
pub(crate) struct Decl<'ctx> {
    pub(crate) identifier: &'ctx str,
    pub(crate) value: &'ctx Expr<'ctx>,
}

#[derive(Clone, Copy)]
pub(crate) enum Expr<'ctx> {
    Const(Const),
    Function(Function<'ctx>),
    If(IfExpr<'ctx>),
    Semi(&'ctx Expr<'ctx>),
}

#[derive(Clone, Copy)]
pub(crate) enum Const {
    IntegerConstant { value: i32 },
}

#[derive(Clone, Copy)]
pub(crate) struct Function<'ctx> {
    pub(crate) return_type: Type,
    pub(crate) parameters: &'ctx [Param<'ctx>],
    pub(crate) body: &'ctx [Expr<'ctx>],
}

#[derive(Clone, Copy)]
pub(crate) struct Param<'ctx> {
    identifier: &'ctx str,
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
    pub(crate) true_branch: &'ctx [Expr<'ctx>],
    pub(crate) else_if_branches: &'ctx [ElseIfBranch<'ctx>],
    pub(crate) final_branch: Option<&'ctx [Expr<'ctx>]>,
}

#[derive(Clone, Copy)]
pub(crate) struct ElseIfBranch<'ctx> {
    pub(crate) cond_expr: &'ctx Expr<'ctx>,
    pub(crate) true_branch: &'ctx [Expr<'ctx>],
}
