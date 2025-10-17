// src/parser.rs

use crate::lexer::{Token, TokenKind, Span}; // Removed unused LexerError import
use serde::Serialize;
use std::iter::Peekable;
use std::vec;

// ... (AST definitions remain exactly the same) ...
#[derive(Debug, Serialize)]
pub struct Program {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Serialize)]
pub enum Statement {
    Read(ReadDirective),
    Write(WriteDirective),
    Iterate(IterationBlock),
    Rule(Rule),
    Fact(Fact),
}

#[derive(Debug, Serialize)]
pub struct ReadDirective {
    pub name: Identifier,
    pub columns: Vec<Identifier>,
    pub path: String,
    pub format: String,
}

#[derive(Debug, Serialize)]
pub struct WriteDirective {
    pub name: Identifier,
    pub path: String,
    pub format: String,
}

#[derive(Debug, Serialize)]
pub struct IterationBlock {
    pub rules: Vec<Rule>,
}

#[derive(Debug, Serialize)]
pub struct Rule {
    pub head: Atom,
    pub body: Vec<Literal>,
}

#[derive(Debug, Serialize)]
pub struct Fact {
    pub atom: GroundAtom,
}

#[derive(Debug, Serialize)]
pub enum Literal {
    Positive(Atom),
    Negative(Atom),
    Condition(Expression),
}

#[derive(Debug, Serialize)]
pub struct Atom {
    pub name: Identifier,
    pub terms: Vec<Expression>,
}

#[derive(Debug, Serialize)]
pub struct GroundAtom {
    pub name: Identifier,
    pub values: Vec<Constant>,
}

#[derive(Debug, Serialize)]
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

#[derive(Debug, Serialize)]
pub struct Aggregate {
    pub func: AggregateFunction,
    pub arg: Identifier,
}

#[derive(Debug, Serialize)]
pub enum AggregateFunction { Count, Sum, Min, Max, Avg }

#[derive(Debug, Serialize, Clone, Copy)]
pub enum BinaryOperator { Add, Sub, Mul, Div, Mod, Eq, NotEq, Lt, LtEq, Gt, GtEq }

#[derive(Debug, Serialize, Clone, Copy)]
pub enum UnaryOperator { Neg }

#[derive(Debug, Serialize, Clone, PartialEq, Eq, Hash)]
pub struct Identifier(pub String);

#[derive(Debug, Serialize, Clone, PartialEq)]
pub enum Constant {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
}

// ... (ParserError remains the same) ...
#[derive(Debug)]
pub struct ParserError {
    pub message: String,
    pub span: Span,
}

impl std::fmt::Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Parser Error at Line {}, Col {}-{}: {}",
            self.span.line, self.span.start, self.span.end, self.message
        )
    }
}
impl std::error::Error for ParserError {}

type ParseResult<T> = Result<T, ParserError>;

pub struct Parser<'a> {
    tokens: Peekable<vec::IntoIter<Token>>,
    source: &'a str,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str, tokens: Vec<Token>) -> Self {
        Parser {
            tokens: tokens.into_iter().peekable(),
            source,
        }
    }

    pub fn parse_program(&mut self) -> ParseResult<Program> {
        let mut statements = Vec::new();
        while self.peek().is_some() {
            statements.push(self.parse_statement()?);
        }
        Ok(Program { statements })
    }

    // ==========================================================
    // THIS IS THE CORRECTED FUNCTION
    // ==========================================================
    fn parse_statement(&mut self) -> ParseResult<Statement> {
        let token = self.peek()
            .cloned() // This is the key change!
            .ok_or_else(|| self.eof_error("Expected a statement"))?;
        
        match &token.kind {
            TokenKind::Read => self.parse_read_directive().map(Statement::Read),
            TokenKind::Write => self.parse_write_directive().map(Statement::Write),
            TokenKind::Iterate => self.parse_iteration_block().map(Statement::Iterate),
            TokenKind::Identifier(_) => {
                let is_rule = self.tokens.clone().any(|t| t.kind == TokenKind::ColonDash);
                if is_rule {
                    self.parse_rule().map(Statement::Rule)
                } else {
                    self.parse_fact().map(Statement::Fact)
                }
            }
            _ => Err(self.unexpected_token_error(&token, "a statement keyword or identifier")),
        }
    }
    
    // ... (All other parser functions remain the same) ...
    fn parse_read_directive(&mut self) -> ParseResult<ReadDirective> {
        self.expect(TokenKind::Read)?;
        let name = self.parse_identifier()?;
        self.expect(TokenKind::LParen)?;
        let columns = self.parse_identifier_list()?;
        self.expect(TokenKind::RParen)?;
        self.expect(TokenKind::From)?;
        let path = self.parse_string_literal()?;
        self.expect(TokenKind::As)?;
        let format = self.parse_string_literal()?;
        self.expect(TokenKind::Dot)?;
        Ok(ReadDirective { name, columns, path, format })
    }

    fn parse_write_directive(&mut self) -> ParseResult<WriteDirective> {
        self.expect(TokenKind::Write)?;
        let name = self.parse_identifier()?;
        self.expect(TokenKind::To)?;
        let path = self.parse_string_literal()?;
        self.expect(TokenKind::As)?;
        let format = self.parse_string_literal()?;
        self.expect(TokenKind::Dot)?;
        Ok(WriteDirective { name, path, format })
    }

    fn parse_iteration_block(&mut self) -> ParseResult<IterationBlock> {
        self.expect(TokenKind::Iterate)?;
        self.expect(TokenKind::LBrace)?;
        let mut rules = Vec::new();
        while self.peek_is_not(&TokenKind::RBrace)? {
            rules.push(self.parse_rule()?);
        }
        self.expect(TokenKind::RBrace)?;
        Ok(IterationBlock { rules })
    }
    
    fn parse_rule(&mut self) -> ParseResult<Rule> {
        let head = self.parse_atom()?;
        self.expect(TokenKind::ColonDash)?;
        let mut body = Vec::new();
        body.push(self.parse_literal()?);
        while self.peek_is(&TokenKind::Comma)? {
            self.consume(); // Consume comma
            body.push(self.parse_literal()?);
        }
        self.expect(TokenKind::Dot)?;
        Ok(Rule { head, body })
    }

    fn parse_fact(&mut self) -> ParseResult<Fact> {
        let atom = self.parse_ground_atom()?;
        self.expect(TokenKind::Dot)?;
        Ok(Fact { atom })
    }
    
    fn parse_literal(&mut self) -> ParseResult<Literal> {
        let is_negated = self.peek_is(&TokenKind::Not)? || self.peek_is(&TokenKind::Bang)?;
        if is_negated {
            self.consume();
            return Ok(Literal::Negative(self.parse_atom()?));
        }

        let is_atom = {
            let mut tokens = self.tokens.clone();
            if let Some(tok1) = tokens.next() {
                if let (TokenKind::Identifier(_), Some(tok2)) = (&tok1.kind, tokens.next()) {
                    matches!(tok2.kind, TokenKind::LParen)
                } else { false }
            } else { false }
        };

        if is_atom {
            Ok(Literal::Positive(self.parse_atom()?))
        } else {
            Ok(Literal::Condition(self.parse_expression()?))
        }
    }

    fn parse_atom(&mut self) -> ParseResult<Atom> {
        let name = self.parse_identifier()?;
        self.expect(TokenKind::LParen)?;
        let terms = if self.peek_is_not(&TokenKind::RParen)? {
            self.parse_list(Self::parse_expression)?
        } else { Vec::new() };
        self.expect(TokenKind::RParen)?;
        Ok(Atom { name, terms })
    }

    fn parse_ground_atom(&mut self) -> ParseResult<GroundAtom> {
        let name = self.parse_identifier()?;
        self.expect(TokenKind::LParen)?;
        let values = if self.peek_is_not(&TokenKind::RParen)? {
            self.parse_list(Self::parse_constant)?
        } else { Vec::new() };
        self.expect(TokenKind::RParen)?;
        Ok(GroundAtom { name, values })
    }
        
    fn parse_expression(&mut self) -> ParseResult<Expression> {
        self.parse_comparison()
    }

    fn parse_comparison(&mut self) -> ParseResult<Expression> {
        let mut expr = self.parse_additive()?;
        while let Some(op_token) = self.peek().cloned() {
            let op = match op_token.kind {
                TokenKind::Eq => BinaryOperator::Eq,
                TokenKind::NotEq => BinaryOperator::NotEq,
                TokenKind::Lt => BinaryOperator::Lt,
                TokenKind::LtEq => BinaryOperator::LtEq,
                TokenKind::Gt => BinaryOperator::Gt,
                TokenKind::GtEq => BinaryOperator::GtEq,
                _ => break,
            };
            self.consume();
            let right = self.parse_additive()?;
            expr = Expression::Binary { left: Box::new(expr), op, right: Box::new(right) };
        }
        Ok(expr)
    }

    fn parse_additive(&mut self) -> ParseResult<Expression> {
        let mut expr = self.parse_multiplicative()?;
        while let Some(op_token) = self.peek().cloned() {
            let op = match op_token.kind {
                TokenKind::Plus => BinaryOperator::Add,
                TokenKind::Minus => BinaryOperator::Sub,
                _ => break,
            };
            self.consume();
            let right = self.parse_multiplicative()?;
            expr = Expression::Binary { left: Box::new(expr), op, right: Box::new(right) };
        }
        Ok(expr)
    }

    fn parse_multiplicative(&mut self) -> ParseResult<Expression> {
        let mut expr = self.parse_unary()?;
        while let Some(op_token) = self.peek().cloned() {
            let op = match op_token.kind {
                TokenKind::Star => BinaryOperator::Mul,
                TokenKind::Slash => BinaryOperator::Div,
                TokenKind::Percent => BinaryOperator::Mod,
                _ => break,
            };
            self.consume();
            let right = self.parse_unary()?;
            expr = Expression::Binary { left: Box::new(expr), op, right: Box::new(right) };
        }
        Ok(expr)
    }
    
    fn parse_unary(&mut self) -> ParseResult<Expression> {
        if self.peek_is(&TokenKind::Minus)? {
            self.consume();
            let expr = self.parse_unary()?;
            Ok(Expression::Unary { op: UnaryOperator::Neg, expr: Box::new(expr) })
        } else {
            self.parse_primary()
        }
    }
    
    fn parse_primary(&mut self) -> ParseResult<Expression> {
        let token = self.consume().ok_or_else(|| self.eof_error("Expected a primary expression"))?;
        match token.kind {
            TokenKind::Integer(i) => Ok(Expression::Constant(Constant::Integer(i))),
            TokenKind::Float(f) => Ok(Expression::Constant(Constant::Float(f))),
            TokenKind::String(s) => Ok(Expression::Constant(Constant::String(s))),
            TokenKind::Boolean(b) => Ok(Expression::Constant(Constant::Boolean(b))),
            TokenKind::Identifier(name) => {
                if self.peek_is(&TokenKind::LParen)? {
                    self.consume();
                    let arg = self.parse_identifier()?;
                    self.expect(TokenKind::RParen)?;
                    let func = match name.as_str() {
                        "count" => AggregateFunction::Count,
                        "sum" => AggregateFunction::Sum,
                        "min" => AggregateFunction::Min,
                        "max" => AggregateFunction::Max,
                        "avg" => AggregateFunction::Avg,
                        _ => return Err(ParserError { message: format!("Unknown aggregate function '{}'", name), span: token.span })
                    };
                    Ok(Expression::Aggregate(Aggregate { func, arg }))
                } else {
                    Ok(Expression::Variable(Identifier(name)))
                }
            }
            TokenKind::Wildcard => Ok(Expression::Wildcard),
            TokenKind::LParen => {
                let expr = self.parse_expression()?;
                self.expect(TokenKind::RParen)?;
                Ok(Expression::Paren(Box::new(expr)))
            },
            _ => Err(self.unexpected_token_error(&token, "a literal, identifier, or expression")),
        }
    }
    
    fn parse_identifier_list(&mut self) -> ParseResult<Vec<Identifier>> {
        self.parse_list(Self::parse_identifier)
    }

    fn parse_list<T, F>(&mut self, mut parse_fn: F) -> ParseResult<Vec<T>>
    where F: FnMut(&mut Self) -> ParseResult<T> {
        let mut items = Vec::new();
        items.push(parse_fn(self)?);
        while self.peek_is(&TokenKind::Comma)? {
            self.consume();
            items.push(parse_fn(self)?);
        }
        Ok(items)
    }

    fn parse_identifier(&mut self) -> ParseResult<Identifier> {
        let token = self.consume().ok_or_else(|| self.eof_error("Expected an identifier"))?;
        if let TokenKind::Identifier(name) = token.kind {
            Ok(Identifier(name))
        } else {
            Err(self.unexpected_token_error(&token, "an identifier"))
        }
    }

    fn parse_string_literal(&mut self) -> ParseResult<String> {
        let token = self.consume().ok_or_else(|| self.eof_error("Expected a string literal"))?;
        if let TokenKind::String(s) = token.kind {
            Ok(s)
        } else {
            Err(self.unexpected_token_error(&token, "a string literal"))
        }
    }

    fn parse_constant(&mut self) -> ParseResult<Constant> {
        let token = self.consume().ok_or_else(|| self.eof_error("Expected a constant"))?;
        match token.kind {
            TokenKind::Integer(i) => Ok(Constant::Integer(i)),
            TokenKind::Float(f) => Ok(Constant::Float(f)),
            TokenKind::String(s) => Ok(Constant::String(s)),
            TokenKind::Boolean(b) => Ok(Constant::Boolean(b)),
            _ => Err(self.unexpected_token_error(&token, "a constant value (integer, string, etc.)")),
        }
    }

    fn peek(&mut self) -> Option<&Token> { self.tokens.peek() }
    fn consume(&mut self) -> Option<Token> { self.tokens.next() }
    
    fn expect(&mut self, expected: TokenKind) -> ParseResult<Token> {
        let token = self.consume().ok_or_else(|| self.eof_error(&format!("Expected '{:?}'", expected)))?;
        if std::mem::discriminant(&token.kind) == std::mem::discriminant(&expected) {
            Ok(token)
        } else {
            Err(self.unexpected_token_error(&token, &format!("'{:?}'", expected)))
        }
    }
    
    fn peek_is(&mut self, kind: &TokenKind) -> ParseResult<bool> {
        Ok(self.peek().map_or(false, |t| std::mem::discriminant(&t.kind) == std::mem::discriminant(kind)))
    }

    fn peek_is_not(&mut self, kind: &TokenKind) -> ParseResult<bool> {
        Ok(self.peek().map_or(false, |t| std::mem::discriminant(&t.kind) != std::mem::discriminant(kind)))
    }
        
    fn eof_error(&self, message: &str) -> ParserError {
        ParserError { message: message.to_string(), span: Span { line: 0, start: 0, end: 0 } }
    }

    fn unexpected_token_error(&self, token: &Token, expected: &str) -> ParserError {
        ParserError {
            message: format!("Unexpected token '{:?}', expected {}", token.kind, expected),
            span: token.span,
        }
    }
}