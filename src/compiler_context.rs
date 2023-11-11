use std::cell::RefCell;

use bumpalo::Bump;

use crate::ast::{Decl, ElseIfBranch, Expr, Param};
use crate::interner::{StringInterner, Symbol};

pub(crate) struct CompilerContext {
    source_code: String,
    string_interner: RefCell<StringInterner>,
    exprs: Bump,
    else_if_branches: Bump,
    params: Bump,
    decls: Bump,
}

#[derive(Clone, Copy)]
pub(crate) struct Span {
    pub(crate) start: BytePos,
    pub(crate) end: BytePos,
}

impl Span {
    pub(crate) fn dummy() -> Span {
        Span {
            start: BytePos::dummy(),
            end: BytePos::dummy(),
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct BytePos(pub(crate) usize);

impl BytePos {
    pub(crate) fn dummy() -> BytePos {
        BytePos(usize::MAX)
    }
}

impl<'ctx> CompilerContext {
    pub(crate) fn new(source_code: String) -> CompilerContext {
        CompilerContext {
            source_code,
            string_interner: Default::default(),
            exprs: Default::default(),
            else_if_branches: Default::default(),
            params: Default::default(),
            decls: Default::default(),
        }
    }

    pub(crate) fn get_source_code(&'ctx self) -> &str {
        &self.source_code
    }

    pub(crate) fn get_text_snippet(&'ctx self, span: Span) -> &'ctx str {
        &self.get_source_code()[span.start.0..span.end.0]
    }

    pub(crate) fn get_or_intern_str(&'ctx self, string: &str) -> Symbol {
        self.string_interner.borrow_mut().get_or_intern(string)
    }

    pub(crate) fn resolve_symbol(&'ctx self, symbol: Symbol) -> &'static str {
        self.string_interner.borrow().resolve(symbol)
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
        params: &'a [Param],
    ) -> &'ctx [Param] {
        self.params.alloc_slice_copy(params)
    }
}
