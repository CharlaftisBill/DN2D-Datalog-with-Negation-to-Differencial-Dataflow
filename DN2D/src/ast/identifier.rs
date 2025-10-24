use serde::Serialize;

use crate::{ast::{parser::ParseResult, Parsable, Parser}, lexer::TokenKind};

#[derive(Debug, Serialize, Clone, PartialEq, Eq, Hash)]
pub struct Identifier(pub String);


impl Parsable<Identifier> for Identifier {
    fn parse(parser :&mut Parser<'_>) -> ParseResult<Identifier> {

        let token = parser.consume().ok_or_else(
            || parser.eof_error("Expected an identifier")
        )?;

        if let TokenKind::Identifier(name) = token.kind {
            Ok(Identifier(name))
        } else {
            Err(parser.unexpected_token_error(&token, " an identifier"))
        }
    }
}