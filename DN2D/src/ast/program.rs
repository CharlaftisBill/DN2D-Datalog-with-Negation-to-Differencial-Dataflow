use serde::Serialize;

use crate::ast::{parser::ParseResult, Parsable, Parser, Statement};

#[derive(Debug, Serialize)]
pub struct Program {
    pub statements: Vec<Statement>,
}

impl Parsable<Program> for Program{
    fn parse(parser :&mut Parser<'_>) -> ParseResult<Program> {
        let mut statements = Vec::new();

        while parser.peek().is_some() {
            statements.push(Statement::parse(parser)?);
        }

        Ok(Program { statements })
    }
}