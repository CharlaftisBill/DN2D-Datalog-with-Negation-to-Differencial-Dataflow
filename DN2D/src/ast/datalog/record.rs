use serde::Serialize;
use crate::{ast::datalog::{parser::ParseResult, Identifier, Parsable, Parser}, lexer::TokenKind};

#[derive(Debug, Serialize, Clone)]
pub enum RecordFieldKind {
    String,
    Integer,
    Boolean,
    Float
}

#[derive(Debug, Serialize, Clone)]
pub struct RecordField {
    pub name: Identifier,
    pub kind: RecordFieldKind,
}

impl Parsable<RecordField> for  RecordField {
    fn parse(parser :&mut Parser<'_>) -> ParseResult<RecordField> {

        let name = Identifier::parse(parser)?;
        let kind = RecordFieldKind::String;
        
        Ok(RecordField { name,  kind})   
    }
}


impl std::fmt::Display for RecordField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            RecordFieldKind::String   => write!(f, "String"), 
            RecordFieldKind::Integer   => write!(f, "i64"),
            RecordFieldKind::Boolean  => write!(f, "bool"),
            RecordFieldKind::Float => write!(f, "ordered_float::OrderedFloat<f64>")
        }
    }
}
