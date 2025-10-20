use serde::Serialize;

use crate::{ast::{parser::ParseResult, GroundAtom, Parsable, Parser}, lexer::TokenKind};

#[derive(Debug, Serialize)]
pub struct Fact {
    pub atom: GroundAtom,
}

impl Parsable<Fact> for Fact{
    fn parse(parser :&mut Parser<'_>) -> ParseResult<Fact> {

        let atom = GroundAtom::parse(parser)?;
        parser.expect(TokenKind::Dot)?;

        Ok(Fact { atom })
    }
}