use serde::Serialize;

use crate::{ast::{parser::ParseResult, Atom, Literal, Parsable, Parser}, lexer::TokenKind};

#[derive(Debug, Serialize)]
pub struct Rule {
    pub head: Atom,
    pub body: Vec<Literal>,
}

impl Parsable<Rule> for Rule{
    fn parse(parser :&mut Parser<'_>) -> ParseResult<Rule> {
        
        let head = Atom::parse(parser)?;
        parser.expect(TokenKind::ColonDash)?;
        
        let mut body = Vec::new();
        body.push(Literal::parse(parser)?);
        
        while parser.peek_is(&TokenKind::Comma)? {
            parser.consume(); // Consume comma
            body.push(Literal::parse(parser)?);
        }
        parser.expect(TokenKind::Dot)?;

        Ok(Rule { head, body })
    }
}