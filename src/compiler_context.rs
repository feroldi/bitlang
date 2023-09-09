use crate::ast::{Decl, ElseIfBranch, Expr, Param};
use bumpalo::Bump;

pub(crate) struct CompilerContext {
    source_code: String,
    identifiers: Bump,
    exprs: Bump,
    if_exprs: Bump,
    else_if_branches: Bump,
    params: Bump,
    decls: Bump,
    consts: Bump,
}

impl<'ctx> CompilerContext {
    pub(crate) fn new(source_code: String) -> CompilerContext {
        CompilerContext {
            source_code,
            identifiers: Default::default(),
            exprs: Default::default(),
            if_exprs: Default::default(),
            else_if_branches: Default::default(),
            params: Default::default(),
            decls: Default::default(),
            consts: Default::default(),
        }
    }

    pub(crate) fn get_source_code(&self) -> &str {
        &self.source_code
    }

    pub(crate) fn intern_ident<'a>(&'ctx self, s: &'a str) -> &'ctx str {
        self.identifiers.alloc_str(s)
    }

    pub(crate) fn alloc_slice_of_decl<'a>(
        &'ctx self,
        decls: &'a [Decl<'ctx>],
    ) -> &'ctx [Decl<'ctx>] {
        self.decls.alloc_slice_copy(decls)
    }

    pub(crate) fn alloc_expr(&'ctx self, expr: Expr<'ctx>) -> &'ctx Expr<'ctx> {
        self.exprs.alloc(expr)
    }

    pub(crate) fn alloc_slice_of_expr<'a>(
        &'ctx self,
        exprs: &'a [Expr<'ctx>],
    ) -> &'ctx [Expr<'ctx>] {
        self.exprs.alloc_slice_copy(exprs)
    }

    pub(crate) fn alloc_slice_of_else_if_branch<'a>(
        &'ctx self,
        else_if_branches: &'a [ElseIfBranch<'ctx>],
    ) -> &'ctx [ElseIfBranch<'ctx>] {
        self.else_if_branches.alloc_slice_copy(else_if_branches)
    }

    pub(crate) fn alloc_slice_of_param<'a>(
        &'ctx self,
        params: &'a [Param<'ctx>],
    ) -> &'ctx [Param<'ctx>] {
        self.params.alloc_slice_copy(params)
    }
}
