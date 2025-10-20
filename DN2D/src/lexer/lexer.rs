use std::{iter::Peekable, str::Chars};

use crate::lexer::{LexerError, Span, Token, TokenKind};

pub struct Lexer<'a> {
    source: &'a str,
    chars: Peekable<Chars<'a>>,
    pos: usize,
    line: usize,
    line_start_pos: usize,
}

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

        let start_col = self.column();
        let start_pos = self.position();

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
            ':' => match self.chars.peek() {
                Some('-') => {
                    self.next_char();
                    TokenKind::ColonDash
                },
                _ => TokenKind::Illegal,
            },
            '!' => match self.chars.peek() {
                Some('=') => {
                    self.next_char();
                    TokenKind::NotEq
                },
                _ => TokenKind::Bang,
            },
            '=' => match self.chars.peek() {
                Some('=') => {
                    self.next_char();
                    TokenKind::Eq
                },
                _ => TokenKind::Illegal,
            },
            '<' => match self.chars.peek() {
                Some('=') => {
                    self.next_char();
                    TokenKind::LtEq
                },
                _ => TokenKind::Lt,
            },
            '>' => match self.chars.peek() {
                Some('=') => {
                    self.next_char();
                    TokenKind::GtEq
                },
                _ => TokenKind::Gt,
            },
            '.' => match self.chars.peek() {
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
            let message = format!("Unrecognized character '{}'", &self.source[start_pos..self.position()]);
            Err(LexerError { message, span })
        } else {
            Ok(Token::new(kind, span))
        }
    }

    fn next_char(&mut self) -> Option<char> {
        self.pos += 1;
        self.chars.next()
    }

    fn position(&mut self) -> usize {
        self.pos
    }

    fn column(&self) -> usize {
        self.pos - self.line_start_pos + 1
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            match self.chars.peek() {
                Some(&c) if c.is_whitespace() => {
                    self.next_char();
                    if c == '\n' {
                        self.line += 1;
                        self.line_start_pos = self.position();
                    }
                }
                Some(&'#') => {
                    self.next_char();
                    while let Some(&c) = self.chars.peek() {
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

        while let Some(&c) = self.chars.peek() {
            if c.is_alphanumeric() || c == '_' {
                ident.push(self.next_char().unwrap());
            } else {
                break;
            }
        }

        match ident.as_str() {
            "from"  => TokenKind::From,
            "to"    => TokenKind::To,
            "as"    => TokenKind::As,
            "not"   => TokenKind::Not,
            "true"  => TokenKind::Boolean(true),
            "false" => TokenKind::Boolean(false),
            _       => TokenKind::Identifier(ident)
        }
    }

    fn read_directive(&mut self) -> Result<TokenKind, LexerError> {
        
        // The mutable borrow on `self` for next_char() ends here (unwrap).
        let first_char = self.next_char().unwrap();

        // New mutable borrow starts here.
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
        
        while let Some(&c) = self.chars.peek() {
            if c.is_digit(10) {
                num_str.push(self.next_char().unwrap());
            } else {
                break;
            }
        }
        
        if let Some('.') = self.chars.peek() {
            if let Some(next_c) = self.source.chars().nth(self.position() + 1) {
                 if next_c.is_digit(10) {
                    num_str.push(self.next_char().unwrap());

                    while let Some(&c) = self.chars.peek() {
                        if c.is_digit(10) {
                            num_str.push(self.next_char().unwrap());
                        } else {
                            break;
                        }
                    }
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