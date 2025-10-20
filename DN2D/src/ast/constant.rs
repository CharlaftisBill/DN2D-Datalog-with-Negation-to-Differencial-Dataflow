use serde::Serialize;

use crate::{ast::{parser::ParseResult, Parsable, Parser}, lexer::TokenKind};

#[derive(Debug, Serialize, Clone, PartialEq)]
pub enum Constant {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
}

impl Parsable<Constant> for Constant{
    fn parse(parser :&mut Parser<'_>) -> ParseResult<Constant> {

        let token = parser.consume().ok_or_else(
            || parser.eof_error("Expected a constant")
        )?;
        
        match token.kind {
            TokenKind::Integer(i) => Ok(Constant::Integer(i)),
            TokenKind::Float(f) => Ok(Constant::Float(f)),
            TokenKind::String(s) => Ok(Constant::String(s)),
            TokenKind::Boolean(b) => Ok(Constant::Boolean(b)),
            _ => Err(parser.unexpected_token_error(&token, "a constant value (integer, string, etc.)")),
        }
    }
}