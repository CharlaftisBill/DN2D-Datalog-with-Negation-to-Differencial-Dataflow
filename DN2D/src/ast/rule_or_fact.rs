use serde::Serialize;

use crate::{ast::{parser::ParseResult, Atom, Literal, Parsable, Parser}, lexer::TokenKind};

#[derive(Debug, Serialize)]
pub enum RuleOrFact {
    Rule(Rule),
    Fact(Fact),
}

#[derive(Debug, Serialize)]
pub struct Rule {
    pub head: Atom,
    pub body: Vec<Literal>,
}

#[derive(Debug, Serialize)]
pub struct Fact {
    pub head: Atom,
}

impl Parsable<RuleOrFact> for RuleOrFact{
    fn parse(parser :&mut Parser<'_>) -> ParseResult<RuleOrFact> {
        
        let head = Atom::parse(parser)?;
        let token = parser.consume().unwrap();
        
        let is_colon_dash = token.kind == TokenKind::ColonDash;
        let is_dot = token.kind == TokenKind::Dot;

        if !is_colon_dash && !is_dot {
            return Err(parser.unexpected_token_error(
                    &token,
                    &format!("'{:?} or {:?}'", TokenKind::ColonDash, TokenKind::Dot)
            ));
        }
        
        let mut body = Vec::new();
        if is_colon_dash {
            body.push(Literal::parse(parser)?);
            
            while parser.peek_is(&TokenKind::Comma)? {
                parser.consume();
                body.push(Literal::parse(parser)?);
            }

            parser.expect(TokenKind::Dot)?;
            return Ok(RuleOrFact::Rule(Rule { head, body }));
        }

        if !is_dot {
            return Err(
                parser.unexpected_token_error(
                    &token,
                    &format!("'{:?}'", TokenKind::Dot)
                )
            );
        }

        Ok(RuleOrFact::Fact(Fact { head }))
    }
}