use crate::ast::*;
use crate::compiler_context::CompilerContext;
use crate::scanner::{Delim, Keyword, Token, TokenKind};

pub(crate) struct Parser<'ctx> {
    ctx: &'ctx CompilerContext,
    tokens: Vec<Token>,
    current_token_idx: usize,
}

impl<'ctx> Parser<'ctx> {
    pub(crate) fn new(tokens: Vec<Token>, ctx: &'ctx CompilerContext) -> Parser {
        Parser {
            ctx,
            tokens,
            current_token_idx: 0,
        }
    }

    pub(crate) fn parse_program(&mut self) -> Option<Program> {
        let mut decls = vec![];

        while let Some(decl) = self.parse_decl() {
            decls.push(decl);
        }

        Some(Program {
            decls: self.ctx.alloc_slice_of_decl(&decls),
        })
    }

    fn parse_decl(&mut self) -> Option<Decl<'ctx>> {
        let ident_tok = self.consume()?;
        debug_assert_eq!(ident_tok.kind, TokenKind::Identifier);

        let op = self.consume()?;
        debug_assert_eq!(op.kind, TokenKind::ColonColon);

        let expr = self.parse_statement_expr()?;

        let identifier = self.ctx.get_or_intern_str(
            &self.ctx.get_source_code()[ident_tok.span.start.0..ident_tok.span.end.0],
        );

        Some(Decl {
            identifier,
            value: self.ctx.alloc_expr(expr),
        })
    }

    fn parse_statement_expr(&mut self) -> Option<Expr<'ctx>> {
        let tok = self.consume()?;

        match tok.kind {
            TokenKind::IntegerConstant => {
                let expr = Expr::Const(Const::IntegerConstant {
                    value: self.ctx.get_source_code()[tok.span.start.0..tok.span.end.0]
                        .parse::<i32>()
                        .unwrap(),
                });

                Some(expr)
            }
            TokenKind::Keyword(Keyword::If) => self.parse_if_expr(),
            TokenKind::Open(Delim::Paren) => self.parse_function(),
            TokenKind::Identifier => {
                if self.peek()?.kind == TokenKind::ColonEqual {
                    self.consume()?;
                    let value = self.parse_statement_expr()?;

                    let identifier = self.ctx.get_or_intern_str(
                        &self.ctx.get_source_code()[tok.span.start.0..tok.span.end.0],
                    );

                    Some(Expr::BindDef(BindDef {
                        identifier,
                        value: self.ctx.alloc_expr(value),
                    }))
                } else {
                    let identifier = self.ctx.get_or_intern_str(
                        &self.ctx.get_source_code()[tok.span.start.0..tok.span.end.0],
                    );

                    Some(Expr::BindRef(BindRef {
                        identifier,
                    }))
                }
            }
            _ => None,
        }
    }

    fn parse_expr(&mut self) -> Option<Expr<'ctx>> {
        let stmt_expr = self.parse_statement_expr()?;

        if self.peek()?.kind == TokenKind::Semi {
            self.consume()?;

            Some(Expr::Semi(self.ctx.alloc_expr(stmt_expr)))
        } else {
            Some(stmt_expr)
        }
    }

    fn parse_if_expr(&mut self) -> Option<Expr<'ctx>> {
        let cond_expr = self.parse_expr()?;

        let open_curly = self.consume()?;
        debug_assert_eq!(open_curly.kind, TokenKind::Open(Delim::Curly));

        let true_branch = self.parse_compound_exprs()?;

        let closed_curly = self.consume()?;
        debug_assert_eq!(closed_curly.kind, TokenKind::Closed(Delim::Curly));

        let mut else_if_branches = vec![];

        while self.peek()?.kind == TokenKind::Keyword(Keyword::Else) {
            if self.look_ahead(1)?.kind != TokenKind::Keyword(Keyword::If) {
                break;
            }

            self.consume()?;
            self.consume()?;

            let cond_expr = self.parse_expr()?;

            let open_curly = self.consume()?;
            debug_assert_eq!(open_curly.kind, TokenKind::Open(Delim::Curly));

            let true_branch = self.parse_compound_exprs()?;

            let closed_curly = self.consume()?;
            debug_assert_eq!(closed_curly.kind, TokenKind::Closed(Delim::Curly));

            else_if_branches.push(ElseIfBranch {
                cond_expr: self.ctx.alloc_expr(cond_expr),
                true_branch,
            });
        }

        let final_branch = if self.peek()?.kind == TokenKind::Keyword(Keyword::Else) {
            self.consume()?;

            let open_curly = self.consume()?;
            debug_assert_eq!(open_curly.kind, TokenKind::Open(Delim::Curly));

            let branch = self.parse_compound_exprs()?;

            let closed_curly = self.consume()?;
            debug_assert_eq!(closed_curly.kind, TokenKind::Closed(Delim::Curly));

            Some(branch)
        } else {
            None
        };

        Some(Expr::If(IfExpr {
            cond_expr: self.ctx.alloc_expr(cond_expr),
            true_branch,
            else_if_branches: self.ctx.alloc_slice_of_else_if_branch(&else_if_branches),
            final_branch,
        }))
    }

    fn parse_function(&mut self) -> Option<Expr<'ctx>> {
        let closed_paren = self.consume()?;
        debug_assert_eq!(closed_paren.kind, TokenKind::Closed(Delim::Paren));

        let next_tok = self.consume()?;

        let return_type = if next_tok.kind == TokenKind::DashGreater {
            let type_tok = self.consume()?;
            debug_assert_eq!(type_tok.kind, TokenKind::Keyword(Keyword::I32));

            let open_curly = self.consume()?;
            debug_assert_eq!(open_curly.kind, TokenKind::Open(Delim::Curly));

            Type::I32
        } else {
            debug_assert_eq!(next_tok.kind, TokenKind::Open(Delim::Curly));

            Type::Unit
        };

        let exprs = self.parse_compound_exprs()?;

        let closed_curly = self.consume()?;
        debug_assert_eq!(closed_curly.kind, TokenKind::Closed(Delim::Curly));

        Some(Expr::Function(Function {
            return_type,
            parameters: self.ctx.alloc_slice_of_param(&[]),
            body: exprs,
        }))
    }

    fn parse_compound_exprs(&mut self) -> Option<&'ctx [Expr<'ctx>]> {
        let mut exprs = vec![];

        while self.peek()?.kind != TokenKind::Closed(Delim::Curly) {
            let expr = self.parse_expr()?;
            exprs.push(expr);
        }

        Some(self.ctx.alloc_slice_of_expr(&exprs))
    }

    fn peek(&self) -> Option<Token> {
        if self.current_token_idx < self.tokens.len() {
            Some(self.tokens[self.current_token_idx])
        } else {
            None
        }
    }

    fn look_ahead(&self, amount: usize) -> Option<Token> {
        let look_ahead_idx = self.current_token_idx + amount;

        if look_ahead_idx < self.tokens.len() {
            Some(self.tokens[look_ahead_idx])
        } else {
            None
        }
    }

    fn consume(&mut self) -> Option<Token> {
        let peeked_tok = self.peek();

        if self.current_token_idx < self.tokens.len() {
            self.current_token_idx += 1;
        }

        peeked_tok
    }
}
