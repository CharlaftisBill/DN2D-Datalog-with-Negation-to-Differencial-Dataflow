use serde::Serialize;

use crate::{ast::{identifier::Identifier, parser::ParseResult, Aggregate, AggregateFunction, BinaryOperator, Constant, Parsable, Parser, ParserError, UnaryOperator}, lexer::TokenKind};

#[derive(Debug, Serialize, Clone)]
pub enum Expression {
    Constant(Constant),
    Variable(Identifier),
    Wildcard,
    Aggregate(Aggregate),
    Binary {
        left: Box<Expression>,
        op: BinaryOperator,
        right: Box<Expression>,
    },
    Unary {
        op: UnaryOperator,
        expr: Box<Expression>,
    },
    Paren(Box<Expression>),
}

impl Parsable<Expression> for Expression {
    fn parse(parser :&mut Parser<'_>) -> ParseResult<Expression> {
        Expression::parse_comparison(parser)
    }
}

impl Expression{
    fn parse_comparison(parser :&mut Parser<'_>) -> ParseResult<Expression> {

        let mut expr = Expression::parse_additive(parser)?;

        while let Some(op_token) = parser.peek().cloned() {
            let op = match op_token.kind {
                TokenKind::Eq => BinaryOperator::Eq,
                TokenKind::NotEq => BinaryOperator::NotEq,
                TokenKind::Lt => BinaryOperator::Lt,
                TokenKind::LtEq => BinaryOperator::LtEq,
                TokenKind::Gt => BinaryOperator::Gt,
                TokenKind::GtEq => BinaryOperator::GtEq,
                _ => break,
            };

            parser.consume();
            let right = Expression::parse_additive(parser)?;
            expr = Expression::Binary { left: Box::new(expr), op, right: Box::new(right) };
        }
        Ok(expr)
    }

    fn parse_additive(parser :&mut Parser<'_>) -> ParseResult<Expression> {

        let mut expr = Expression::parse_multiplicative(parser)?;

        while let Some(op_token) = parser.peek().cloned() {
            let op = match op_token.kind {
                TokenKind::Plus => BinaryOperator::Add,
                TokenKind::Minus => BinaryOperator::Sub,
                _ => break,
            };

            parser.consume();
            let right = Expression::parse_multiplicative(parser)?;
            expr = Expression::Binary { left: Box::new(expr), op, right: Box::new(right) };
        }
        Ok(expr)
    }

    fn parse_multiplicative(parser :&mut Parser<'_>) -> ParseResult<Expression> {

        let mut expr = Expression::parse_unary(parser)?;

        while let Some(op_token) = parser.peek().cloned() {
            let op = match op_token.kind {
                TokenKind::Star => BinaryOperator::Mul,
                TokenKind::Slash => BinaryOperator::Div,
                TokenKind::Percent => BinaryOperator::Mod,
                _ => break,
            };

            parser.consume();
            let right = Expression::parse_unary(parser)?;
            expr = Expression::Binary { left: Box::new(expr), op, right: Box::new(right) };
        }
        Ok(expr)
    }
    
    fn parse_unary(parser :&mut Parser<'_>) -> ParseResult<Expression> {

        if parser.peek_is(&TokenKind::Minus)? {

            parser.consume();

            let expr = Expression::parse_unary(parser)?;
            Ok(Expression::Unary { op: UnaryOperator::Neg, expr: Box::new(expr) })
        } else {
            Expression::parse_primary(parser)
        }
    }
    
    fn parse_primary(parser :&mut Parser<'_>) -> ParseResult<Expression> {

        let token = parser.consume().ok_or_else(
            || parser.eof_error("Expected a primary expression")
        )?;
        
        match &token.kind {
            TokenKind::Integer(i) => Ok(Expression::Constant(Constant::Integer(*i))),
            TokenKind::Float(f) => Ok(Expression::Constant(Constant::Float(*f))),
            TokenKind::String(s) => Ok(Expression::Constant(Constant::String(s.clone()))),
            TokenKind::Boolean(b) => Ok(Expression::Constant(Constant::Boolean(*b))),
            TokenKind::Identifier(name) => {
                
                if parser.peek_is(&TokenKind::LParen)? {

                    parser.consume();
                    let arg = Identifier::parse(parser)?;

                    parser.expect(TokenKind::RParen)?;

                    let func = match name.as_str() {
                        "count" => AggregateFunction::Count,
                        "sum" => AggregateFunction::Sum,
                        "min" => AggregateFunction::Min,
                        "max" => AggregateFunction::Max,
                        "avg" => AggregateFunction::Avg,
                        _ => return Err(
                            ParserError {
                                message: format!("Unknown aggregate function '{}'", name),
                                line_ref: parser.source_line(&token),
                                span: token.span
                            }
                        )
                    };
                    Ok(Expression::Aggregate(Aggregate { func, arg }))
                } else {
                    Ok(Expression::Variable(Identifier(name.clone())))
                }
            }
            TokenKind::Wildcard => Ok(Expression::Wildcard),
            TokenKind::LParen => {
                let expr = Expression::parse(parser)?;
                parser.expect(TokenKind::RParen)?;
                Ok(Expression::Paren(Box::new(expr)))
            },
            _ => Err(parser.unexpected_token_error(&token, "a literal, identifier, or expression")),
        }
    }
}

