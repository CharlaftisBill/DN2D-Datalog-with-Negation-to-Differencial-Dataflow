use serde::Serialize;

use crate::{ast::{parser::ParseResult, Parsable, Parser, RuleOrFact}, lexer::TokenKind};

#[derive(Debug, Serialize, Clone)]
pub struct IterationBlock {
    pub rules: Vec<RuleOrFact>,
}

impl Parsable<IterationBlock> for IterationBlock {
    fn parse(parser :&mut Parser<'_>) -> ParseResult<IterationBlock> {

        parser.expect(TokenKind::Iterate)?;
        parser.expect(TokenKind::LBrace)?;
        let mut rules = Vec::new();

        while parser.peek_is_not(&TokenKind::RBrace)? {
            rules.push(RuleOrFact::parse(parser)?);
        }
        parser.expect(TokenKind::RBrace)?;
        
        Ok(IterationBlock { rules })
    }
}