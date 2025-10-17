// src/lexer.rs

use std::fmt;
use std::iter::Peekable;
use std::str::Chars;

// ==========================================================
//                  1. DATA STRUCTURES
// ==========================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub line: usize,
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(line: usize, start: usize, end: usize) -> Self {
        Span { line, start, end }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Read, Write, Iterate, As, From, To, Not,
    LParen, RParen, LBrace, RBrace, Comma, Dot, ColonDash, Wildcard,
    Eq, NotEq, Lt, LtEq, Gt, GtEq, Plus, Minus, Star, Slash, Percent, Bang,
    Identifier(String),
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Eof,
    Illegal,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Token { kind, span }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LexerError {
    pub message: String,
    pub span: Span,
}

impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Lexer Error at Line {}, Col {}-{}: {}",
            self.span.line, self.span.start, self.span.end, self.message
        )
    }
}
impl std::error::Error for LexerError {}

// ==========================================================
//                      2. LEXER STRUCT
// ==========================================================

pub struct Lexer<'a> {
    source: &'a str,
    chars: Peekable<Chars<'a>>,
    pos: usize,
    line: usize,
    line_start_pos: usize,
}

// ==========================================================
//                    3. LEXER IMPLEMENTATION
// ==========================================================

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Lexer {
            source,
            chars: source.chars().peekable(),
            pos: 0,
            line: 1,
            line_start_pos: 0,
        }
    }

    fn next_token(&mut self) -> Result<Token, LexerError> {
        self.skip_whitespace_and_comments();
        let start_pos = self.pos;
        let start_col = self.column();

        let Some(ch) = self.next_char() else {
            let span = Span::new(self.line, start_col, start_col);
            return Ok(Token::new(TokenKind::Eof, span));
        };

        let kind = match ch {
            '(' => TokenKind::LParen,
            ')' => TokenKind::RParen,
            '{' => TokenKind::LBrace,
            '}' => TokenKind::RBrace,
            ',' => TokenKind::Comma,
            '_' => TokenKind::Wildcard,
            '+' => TokenKind::Plus,
            '-' => TokenKind::Minus,
            '*' => TokenKind::Star,
            '/' => TokenKind::Slash,
            '%' => TokenKind::Percent,
            ':' => match self.peek() {
                Some('-') => { self.next_char(); TokenKind::ColonDash },
                _ => TokenKind::Illegal,
            },
            '!' => match self.peek() {
                Some('=') => { self.next_char(); TokenKind::NotEq },
                _ => TokenKind::Bang,
            },
            '=' => match self.peek() {
                Some('=') => { self.next_char(); TokenKind::Eq },
                _ => TokenKind::Illegal,
            },
            '<' => match self.peek() {
                Some('=') => { self.next_char(); TokenKind::LtEq },
                _ => TokenKind::Lt,
            },
            '>' => match self.peek() {
                Some('=') => { self.next_char(); TokenKind::GtEq },
                _ => TokenKind::Gt,
            },
            '.' => match self.peek() {
                Some(c) if c.is_alphabetic() => self.read_directive()?,
                _ => TokenKind::Dot,
            }
            '"' => self.read_string()?,
            c if c.is_digit(10) => self.read_number(c)?,
            c if c.is_alphabetic() => self.read_identifier_or_keyword(c),
            _ => TokenKind::Illegal,
        };

        let span = Span::new(self.line, start_col, self.column() - 1);
        if kind == TokenKind::Illegal {
            let message = format!("Unrecognized character '{}'", &self.source[start_pos..self.pos]);
            Err(LexerError { message, span })
        } else {
            Ok(Token::new(kind, span))
        }
    }

    fn next_char(&mut self) -> Option<char> {
        self.pos += 1;
        self.chars.next()
    }

    fn peek(&mut self) -> Option<&char> {
        self.chars.peek()
    }

    fn column(&self) -> usize {
        self.pos - self.line_start_pos + 1
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            match self.peek() {
                Some(&c) if c.is_whitespace() => {
                    self.next_char();
                    if c == '\n' {
                        self.line += 1;
                        self.line_start_pos = self.pos;
                    }
                }
                Some('#') => {
                    self.next_char();
                    while let Some(&c) = self.peek() {
                        if c == '\n' { break; }
                        self.next_char();
                    }
                }
                _ => break,
            }
        }
    }
    
    fn read_identifier_or_keyword(&mut self, first: char) -> TokenKind {
        let mut ident = String::new();
        ident.push(first);
        while let Some(&c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                ident.push(self.next_char().unwrap());
            } else {
                break;
            }
        }
        match ident.as_str() {
            "as" => TokenKind::As, "from" => TokenKind::From, "to" => TokenKind::To,
            "not" => TokenKind::Not, "true" => TokenKind::Boolean(true),
            "false" => TokenKind::Boolean(false), _ => TokenKind::Identifier(ident),
        }
    }

    // ==========================================================
    // THIS IS THE CORRECTED FUNCTION
    // ==========================================================
    fn read_directive(&mut self) -> Result<TokenKind, LexerError> {
        // First, get the character. The mutable borrow on `self` for next_char() ends here.
        let first_char = self.next_char().unwrap();
        // Then, pass the character's value. A new mutable borrow for read_identifier_or_keyword() starts here.
        let directive = self.read_identifier_or_keyword(first_char);
        
        match directive {
            TokenKind::Identifier(s) => match s.as_str() {
                "read" => Ok(TokenKind::Read),
                "write" => Ok(TokenKind::Write),
                "iterate" => Ok(TokenKind::Iterate),
                _ => {
                    let span = Span::new(self.line, self.column() - s.len() - 1, self.column() - 1);
                    Err(LexerError { message: format!("Unknown directive '.{}'", s), span, })
                }
            },
            _ => unreachable!(),
        }
    }
    
    fn read_number(&mut self, first: char) -> Result<TokenKind, LexerError> {
        let mut num_str = String::new();
        num_str.push(first);
        while let Some(&c) = self.peek() { if c.is_digit(10) { num_str.push(self.next_char().unwrap()); } else { break; } }
        if let Some('.') = self.peek() {
            if let Some(next_c) = self.source.chars().nth(self.pos + 1) {
                 if next_c.is_digit(10) {
                    num_str.push(self.next_char().unwrap());
                    while let Some(&c) = self.peek() { if c.is_digit(10) { num_str.push(self.next_char().unwrap()); } else { break; } }
                    return Ok(TokenKind::Float(num_str.parse().unwrap()));
                }
            }
        }
        Ok(TokenKind::Integer(num_str.parse().unwrap()))
    }

    fn read_string(&mut self) -> Result<TokenKind, LexerError> {
        let mut s = String::new();
        let start_col = self.column() - 1;
        loop {
            match self.next_char() {
                Some('"') => break,
                Some(c) => s.push(c),
                None => {
                    let span = Span::new(self.line, start_col, self.column());
                    return Err(LexerError { message: "Unterminated string literal".into(), span, });
                }
            }
        }
        Ok(TokenKind::String(s))
    }
}

// ==========================================================
//                   4. ITERATOR IMPLEMENTATION
// ==========================================================

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<Token, LexerError>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.next_token() {
            Ok(token) if token.kind == TokenKind::Eof => None,
            Ok(token) => Some(Ok(token)),
            Err(e) => Some(Err(e)),
        }
    }
}