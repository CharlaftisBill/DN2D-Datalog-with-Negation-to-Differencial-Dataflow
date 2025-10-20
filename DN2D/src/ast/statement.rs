use serde::Serialize;

use crate::{ast::{parser::ParseResult, Fact, IterationBlock, Parsable, Parser, ReadDirective, Rule, WriteDirective}, lexer::TokenKind};

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
                
                let is_rule = parser.tokens
                    .clone()
                    .any(|t| t.kind == TokenKind::ColonDash);

                if is_rule {
                    Rule::parse(parser)
                        .map(Statement::Rule)
                } else {
                    Fact::parse(parser)
                        .map(Statement::Fact)
                }
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