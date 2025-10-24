use serde::Serialize;

use crate::{ast::{parser::ParseResult, Expression, Identifier, Parsable, Parser}, lexer::TokenKind};

#[derive(Debug, Serialize)]
pub struct Atom {
    pub name: Identifier,
    pub terms: Vec<Expression>,
}

impl Parsable<Atom> for Atom {
    fn parse(parser :&mut Parser<'_>) -> ParseResult<Atom> {
        
        let name = Identifier::parse(parser)?;
        parser.expect(TokenKind::LParen)?;

        let terms = if parser.peek_is_not(&TokenKind::RParen)? {
            parser.parse_list(Expression::parse)?
        } else {
            Vec::new()
        };

        parser.expect(TokenKind::RParen)?;

        Ok(Atom { name, terms })
    }
}