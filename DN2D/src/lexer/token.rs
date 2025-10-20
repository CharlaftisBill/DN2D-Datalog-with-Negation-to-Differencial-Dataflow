use std::fmt;

use super::span::Span;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Read, From,
    Write, To, As,
    Iterate,
    LParen, RParen, LBrace, RBrace, Comma, Dot, ColonDash, Wildcard,
    Not, Eq, NotEq, Lt, LtEq, Gt, GtEq, Plus, Minus, Star, Slash, Percent, Bang,
    Identifier(String),
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Eof,
    Illegal,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Token { kind, span }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LexerError {
    pub message: String,
    pub span: Span,
}

impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Lexer Error at Line {}, Col {}-{}: {}",
            self.span.line, self.span.start, self.span.end, self.message
        )
    }
}

impl std::error::Error for LexerError {}
