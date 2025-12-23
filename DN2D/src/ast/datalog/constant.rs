use serde::Serialize;

use crate::{ast::datalog::{parser::ParseResult, Parsable, Parser}, lexer::TokenKind};

#[derive(Debug, Serialize, Clone, PartialEq)]
pub enum Constant {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
}

impl Constant{
    pub fn value(&self) -> String {
        match self {
            Constant::String(value) => value.clone(), 
            Constant::Integer(value)   => value.to_string(),
            Constant::Boolean(value)  => value.to_string(),
            Constant::Float(value)     => value.to_string()
        }
    }
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

impl std::fmt::Display for Constant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Constant::String(_)   => write!(f, "String"), 
            Constant::Integer(_)   => write!(f, "i64"),
            Constant::Boolean(_)  => write!(f, "bool"),
            Constant::Float(_) => write!(f, "ordered_float::OrderedFloat<f64>")
        }
    }
}