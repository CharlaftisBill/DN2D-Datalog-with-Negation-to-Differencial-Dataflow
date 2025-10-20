use serde::Serialize;

use crate::{ast::{parser::ParseResult, Constant, Identifier, Parsable, Parser, Rule}, lexer::TokenKind};

#[derive(Debug, Serialize)]
pub struct IterationBlock {
    pub rules: Vec<Rule>,
}

impl Parsable<IterationBlock> for IterationBlock {
    fn parse(parser :&mut Parser<'_>) -> ParseResult<IterationBlock> {

        parser.expect(TokenKind::Iterate)?;
        parser.expect(TokenKind::LBrace)?;
        let mut rules = Vec::new();

        while parser.peek_is_not(&TokenKind::RBrace)? {
            rules.push(Rule::parse(parser)?);
        }
        parser.expect(TokenKind::RBrace)?;
        
        Ok(IterationBlock { rules })
    }
}


#[derive(Debug, Serialize)]
pub struct GroundAtom {
    pub name: Identifier,
    pub values: Vec<Constant>,
}

impl Parsable<GroundAtom> for GroundAtom {
    fn parse(parser :&mut Parser<'_>) -> ParseResult<GroundAtom> {
        
        let name = Identifier::parse(parser)?;
        parser.expect(TokenKind::LParen)?;

        let values = if parser.peek_is_not(&TokenKind::RParen)? {
            parser.parse_list(Constant::parse)?
        } else {
            Vec::new()
        };

        parser.expect(TokenKind::RParen)?;
        Ok(GroundAtom { name, values })
    }
}