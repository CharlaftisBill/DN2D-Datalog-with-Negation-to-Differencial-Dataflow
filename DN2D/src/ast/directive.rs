use serde::Serialize;

use crate::{ast::{parser::ParseResult, Identifier, Parsable, Parser}, lexer::TokenKind};

#[derive(Debug, Serialize, Clone)]
pub struct ReadDirective {
    pub name: Identifier,
    pub columns: Vec<Identifier>,
    pub path: String,
    pub format: String,
}

impl Parsable<ReadDirective> for  ReadDirective {
    fn parse(parser :&mut Parser<'_>) -> ParseResult<ReadDirective> {

        parser.expect(TokenKind::Read)?;
        let name = Identifier::parse(parser)?;

        parser.expect(TokenKind::LParen)?;
        let columns =parser.parse_list(Identifier::parse)?;

        parser.expect(TokenKind::RParen)?;
        parser.expect(TokenKind::From)?;
        let path = parser.parse_string_literal()?;

        parser.expect(TokenKind::As)?;
        let format = parser.parse_string_literal()?;

        parser.expect(TokenKind::Dot)?;
        
        Ok(ReadDirective { name, columns, path, format })   
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct WriteDirective {
    pub name: Identifier,
    pub path: String,
    pub format: String,
}

impl Parsable<WriteDirective> for  WriteDirective {
    fn parse(parser :&mut Parser<'_>) -> ParseResult<WriteDirective> {
        
        parser.expect(TokenKind::Write)?;
        let name = Identifier::parse(parser)?;

        parser.expect(TokenKind::To)?;
        let path = parser.parse_string_literal()?;

        parser.expect(TokenKind::As)?;
        let format = parser.parse_string_literal()?;
        
        parser.expect(TokenKind::Dot)?;

        Ok(WriteDirective { name, path, format })
    }
}