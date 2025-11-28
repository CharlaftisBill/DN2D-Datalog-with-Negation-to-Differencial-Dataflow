use serde::Serialize;

use crate::{ast::{parser::ParseResult, Atom, Expression, Parsable, Parser}, lexer::TokenKind};

#[derive(Debug, Serialize, Clone)]
pub enum Literal {
    Positive(Atom),
    Negative(Atom),
    Condition(Expression),
}

impl Parsable<Literal> for Literal{
    fn parse(parser :&mut Parser<'_>) -> ParseResult<Literal> {

        let is_negated = parser.peek_is(&TokenKind::Not)? ||
        parser.peek_is(&TokenKind::Bang)?;
        
        if is_negated {
            parser.consume();
            return Ok(Literal::Negative(Atom::parse(parser)?));
        }
        
        let is_atom = {
            let mut tokens = parser.tokens.clone();
            if let Some(tok1) = tokens.next() {
                if let (TokenKind::Identifier(_), Some(tok2)) = (&tok1.kind, tokens.next()) {
                    matches!(tok2.kind, TokenKind::LParen)
                } else {
                    false
                }
            } else {
                false
            }
        };

        if is_atom {
            Ok(Literal::Positive(Atom::parse(parser)?))
        } else {
            Ok(Literal::Condition(Expression::parse(parser)?))
        }
    }
}