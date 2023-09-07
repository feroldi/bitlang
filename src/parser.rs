use crate::scanner::{Delim, Keyword, Token, TokenKind};

pub(crate) struct Parser<'s> {
    source_code: &'s str,
    tokens: Vec<Token>,
    current_token_idx: usize,
}

impl Parser<'_> {
    pub(crate) fn new(source_code: &str, tokens: Vec<Token>) -> Parser {
        Parser {
            source_code,
            tokens,
            current_token_idx: 0,
        }
    }

    pub(crate) fn parse_program(&mut self) -> Option<Program> {
        let mut decls = vec![];

        while let Some(decl) = self.parse_decl() {
            decls.push(decl);
        }

        Some(Program { decls })
    }

    fn parse_decl(&mut self) -> Option<Decl> {
        let ident_tok = self.consume()?;
        debug_assert_eq!(ident_tok.kind, TokenKind::Identifier);

        let op = self.consume()?;
        debug_assert_eq!(op.kind, TokenKind::ColonColon);

        let expr = self.parse_statement_expr()?;

        Some(Decl {
            identifier: self.source_code[ident_tok.span.start.0..ident_tok.span.end.0].to_owned(),
            value: expr,
        })
    }

    fn parse_statement_expr(&mut self) -> Option<Expr> {
        let tok = self.consume()?;

        match tok.kind {
            TokenKind::IntegerConstant => Some(Expr::Const(Const::IntegerConstant {
                value: self.source_code[tok.span.start.0..tok.span.end.0]
                    .parse::<i32>()
                    .unwrap(),
            })),
            TokenKind::Keyword(Keyword::If) => self.parse_if_expr(),
            TokenKind::Open(Delim::Paren) => self.parse_function().map(Expr::Function),
            _ => None,
        }
    }

    fn parse_expr(&mut self) -> Option<Expr> {
        let stmt_expr = self.parse_statement_expr()?;

        if self.peek()?.kind == TokenKind::Semi {
            self.consume()?;

            Some(Expr::Semi(Box::new(stmt_expr)))
        } else {
            Some(stmt_expr)
        }
    }

    fn parse_if_expr(&mut self) -> Option<Expr> {
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
                cond_expr,
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
            cond_expr: Box::new(cond_expr),
            true_branch,
            else_if_branches,
            final_branch,
        }))
    }

    fn parse_function(&mut self) -> Option<Function> {
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

        Some(Function {
            return_type,
            parameters: vec![],
            body: exprs,
        })
    }

    fn parse_compound_exprs(&mut self) -> Option<Vec<Expr>> {
        let mut exprs = vec![];

        while self.peek()?.kind != TokenKind::Closed(Delim::Curly) {
            let expr = self.parse_expr()?;
            exprs.push(expr);
        }

        Some(exprs)
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

pub(crate) struct Program {
    pub(crate) decls: Vec<Decl>,
}

pub(crate) struct Decl {
    pub(crate) identifier: String,
    pub(crate) value: Expr,
}

pub(crate) enum Expr {
    Identifier(String),
    Const(Const),
    Function(Function),
    If(IfExpr),
    Semi(Box<Expr>),
}

pub(crate) enum Const {
    IntegerConstant { value: i32 },
}

pub(crate) struct Function {
    return_type: Type,
    parameters: Vec<Param>,
    pub(crate) body: Vec<Expr>,
}

pub(crate) struct Param {
    identifier: String,
    ty: Type,
}

pub(crate) enum Type {
    Unit,
    I32,
}

pub(crate) struct IfExpr {
    pub(crate) cond_expr: Box<Expr>,
    pub(crate) true_branch: Vec<Expr>,
    pub(crate) else_if_branches: Vec<ElseIfBranch>,
    pub(crate) final_branch: Option<Vec<Expr>>,
}

pub(crate) struct ElseIfBranch {
    pub(crate) cond_expr: Expr,
    pub(crate) true_branch: Vec<Expr>,
}
