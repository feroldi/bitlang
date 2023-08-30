use std::iter::Peekable;
use std::str::Chars;

pub(crate) struct Scanner<'s> {
    source_code: &'s str,
    char_stream: Peekable<Chars<'s>>,
    current_peek_pos: BytePos,
}

impl Scanner<'_> {
    const EOF_CHAR: char = '\0';

    pub(crate) fn new(source_code: &str) -> Scanner {
        Scanner {
            source_code,
            char_stream: source_code.chars().peekable(),
            current_peek_pos: BytePos(0),
        }
    }

    pub(crate) fn scan_all_tokens(&mut self) -> Vec<Token> {
        let mut tokens = vec![];

        while let Some(token) = self.scan_next_token() {
            tokens.push(token)
        }

        tokens
    }

    fn scan_next_token(&mut self) -> Option<Token> {
        self.skip_whitespace();

        let span_start = self.current_peek_pos;

        let token_kind = match self.bump() {
            Scanner::EOF_CHAR => return None,
            ':' if self.peek() == ':' => {
                self.bump();

                TokenKind::ColonColon
            }
            '(' => TokenKind::Open(Delim::Paren),
            ')' => TokenKind::Closed(Delim::Paren),
            '{' => TokenKind::Open(Delim::Curly),
            '}' => TokenKind::Closed(Delim::Curly),
            '-' if self.peek() == '>' => {
                self.bump();

                TokenKind::DashGreater
            }
            '0'..='9' => self.scan_integer_constant(),
            'a'..='z' | 'A'..='Z' | '_' => self.scan_identifier(span_start),
            ch => todo!("char not recognized: '{}'", ch),
        };

        let token_span = Span {
            start: span_start,
            end: self.current_peek_pos,
        };

        Some(Token {
            kind: token_kind,
            span: token_span,
        })
    }

    fn skip_whitespace(&mut self) {
        while self.peek().is_ascii_whitespace() {
            self.bump();
        }
    }

    fn scan_integer_constant(&mut self) -> TokenKind {
        while self.peek().is_ascii_digit() {
            self.bump();
        }

        TokenKind::IntegerConstant
    }

    fn scan_identifier(&mut self, ident_span_start: BytePos) -> TokenKind {
        while matches!(self.peek(), 'a'..='z' | 'A'..='Z' | '_' | '0'..='9') {
            self.bump();
        }

        let ident_text = &self.source_code[ident_span_start.0..self.current_peek_pos.0];

        match ident_text {
            "i32" => TokenKind::Keyword(Keyword::I32),
            _ => TokenKind::Identifier,
        }
    }

    fn peek(&mut self) -> char {
        self.char_stream
            .peek()
            .cloned()
            .unwrap_or(Scanner::EOF_CHAR)
    }

    fn bump(&mut self) -> char {
        let peeked = self.peek();

        if peeked != Scanner::EOF_CHAR {
            self.current_peek_pos = BytePos(self.current_peek_pos.0 + peeked.len_utf8());
            self.char_stream.next();
        }

        peeked
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Token {
    pub(crate) kind: TokenKind,
    pub(crate) span: Span,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum TokenKind {
    UnitConstant,
    IntegerConstant,
    Identifier,
    Comma,
    Excla,
    Star,
    Slash,
    Plus,
    Dash,
    Less,
    Greater,
    LessLess,
    GreaterGreater,
    LessEqual,
    GreaterEqual,
    ColonColon,
    DashGreater,
    Keyword(Keyword),
    Open(Delim),
    Closed(Delim),
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum Keyword {
    Break,
    Continue,
    I32,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum Delim {
    Paren,
    Curly,
}

#[derive(Clone, Copy)]
pub(crate) struct Span {
    pub(crate) start: BytePos,
    pub(crate) end: BytePos,
}

#[derive(Clone, Copy)]
pub(crate) struct BytePos(pub(crate) usize);
