use crate::ast::{parser::ParseResult, Parser};

pub trait Parsable<T> {
     fn parse(parser :&mut Parser<'_>) -> ParseResult<T>;
}