use std::{iter::Peekable, vec};

use crate::{ast::ParserError, lexer::{Span, Token, TokenKind}};

pub type ParseResult<T> = Result<T, ParserError>;

pub struct Parser<'a> {
    pub(crate) tokens: Peekable<vec::IntoIter<Token>>,
    pub source: &'a str,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str, tokens: Vec<Token>) -> Self {
        Parser {
            tokens: tokens.into_iter().peekable(),
            source,
        }
    }

    pub fn source_line(&self, token: &Token) -> String {
        self.source.lines().nth(token.span.line).unwrap().to_string()
    }

    pub fn parse_list<T, F>(&mut self, mut parse_fn: F) -> ParseResult<Vec<T>>
    where F: FnMut(&mut Self) -> ParseResult<T> {

        let mut items = Vec::new();
        items.push(parse_fn(self)?);

        while self.peek_is(&TokenKind::Comma)? {
            self.consume();
            items.push(parse_fn(self)?);
        }
        
        Ok(items)
    }

    pub fn parse_string_literal(&mut self) -> ParseResult<String> {
        let token = self.consume().ok_or_else(|| self.eof_error("Expected a string literal"))?;
        if let TokenKind::String(s) = token.kind {
            Ok(s)
        } else {
            Err(self.unexpected_token_error(&token, "a string literal"))
        }
    }

    pub fn peek(&mut self) -> Option<&Token> {
        self.tokens.peek()
    }

    pub fn consume(&mut self) -> Option<Token> {
        self.tokens.next()
    }
    
    pub fn expect(&mut self, expected: TokenKind) -> ParseResult<Token> {
        let token = self.consume().ok_or_else(|| self.eof_error(&format!("Expected '{:?}'", expected)))?;
        if std::mem::discriminant(&token.kind) == std::mem::discriminant(&expected) {
            Ok(token)
        } else {
            Err(self.unexpected_token_error(&token, &format!("'{:?}'", expected)))
        }
    }
    
    pub fn peek_is(&mut self, kind: &TokenKind) -> ParseResult<bool> {
        Ok(self.peek().map_or(false, |t| std::mem::discriminant(&t.kind) == std::mem::discriminant(kind)))
    }

    pub fn peek_is_not(&mut self, kind: &TokenKind) -> ParseResult<bool> {
        Ok(self.peek().map_or(false, |t| std::mem::discriminant(&t.kind) != std::mem::discriminant(kind)))
    }
        
    pub fn eof_error(&self, message: &str) -> ParserError {
        ParserError {
            message: message.to_string(),
            line_ref: "".to_string(),
            span: Span {
                line: 0,
                start: 0,
                end: 0
            }
        }
    }

    pub fn unexpected_token_error(&self, token: &Token, expected: &str) -> ParserError {
        ParserError {
            message: format!("Unexpected token '{:?}', expected {}", token.kind, expected),
            line_ref: self.source_line(token),
            span: token.span,
        }
    }
}