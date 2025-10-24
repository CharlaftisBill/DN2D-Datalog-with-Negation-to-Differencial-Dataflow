
use serde::Serialize;

use crate::{ast::{parser::ParseResult, rule_or_fact::{Fact, Rule}, IterationBlock, Parsable, Parser, ReadDirective, RuleOrFact, WriteDirective}, lexer::TokenKind};

#[derive(Debug, Serialize)]
pub enum Statement {
    Read(ReadDirective),
    Write(WriteDirective),
    Iterate(IterationBlock),
    Rule(Rule),
    Fact(Fact),
}

impl Parsable<Statement> for Statement{
    fn parse(parser :&mut Parser<'_>) -> ParseResult<Statement> {

        let token = parser.peek()
            .cloned()
            .ok_or_else(|| parser.eof_error("Expected a statement"))?;
        
        match &token.kind {
            TokenKind::Read => ReadDirective::parse(parser)
                .map(Statement::Read),
            TokenKind::Write => WriteDirective::parse(parser)
                .map(Statement::Write),
            TokenKind::Iterate => IterationBlock::parse(parser)
                .map(Statement::Iterate),
            TokenKind::Identifier(_) => {
                return match RuleOrFact::parse(parser) {
                    Ok(rule_or_fact) => {
                        match  rule_or_fact{
                            RuleOrFact::Rule(rule) => Ok(Statement::Rule(rule)),
                            RuleOrFact::Fact(fact) => Ok(Statement::Fact(fact)),
                        }
                    },
                    Err(e) => Err(e)
                };
            }
            _ => Err(
                parser.unexpected_token_error(
                    &token,
                    "a statement keyword or identifier"
                )
            ),
        }
    }
}