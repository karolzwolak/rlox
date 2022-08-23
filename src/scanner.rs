use std::{iter::Peekable, slice::Iter};

use crate::{token::Token, token::TokenKind, Error, Result};
pub struct Scanner<'a> {
    source: &'a str,
    byte_iter: Peekable<Iter<'a, u8>>,
    start: usize,
    current: usize,
    line: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            byte_iter: source.as_bytes().into_iter().peekable(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_token(&mut self) -> Result<Token> {
        self.skip_whitespace();
        self.start = self.current;

        if let Some(ch) = self.advance() {
            Ok(match ch {
                b'(' => self.make_token(TokenKind::RightParen),
                b')' => self.make_token(TokenKind::LeftParen),
                b'{' => self.make_token(TokenKind::LeftBrace),
                b'}' => self.make_token(TokenKind::RightBrace),
                b',' => self.make_token(TokenKind::Comma),
                b'.' => self.make_token(TokenKind::Dot),
                b'-' => self.make_token(TokenKind::Minus),
                b'+' => self.make_token(TokenKind::Plus),
                b';' => self.make_token(TokenKind::Semicolon),
                b'*' => self.make_token(TokenKind::Star),
                b'!' => {
                    if self.match_next(b'=') {
                        self.make_token(TokenKind::BangEqual)
                    } else {
                        self.make_token(TokenKind::Bang)
                    }
                }
                b'=' => {
                    if self.match_next(b'=') {
                        self.make_token(TokenKind::EqualEqual)
                    } else {
                        self.make_token(TokenKind::Equal)
                    }
                }
                b'<' => {
                    if self.match_next(b'=') {
                        self.make_token(TokenKind::LessEqual)
                    } else {
                        self.make_token(TokenKind::Less)
                    }
                }
                b'>' => {
                    if self.match_next(b'=') {
                        self.make_token(TokenKind::GreaterEqual)
                    } else {
                        self.make_token(TokenKind::Greater)
                    }
                }
                b'"' => return self.make_string(),

                b'0'..=b'9' => return self.make_number(),

                b'a'..=b'z' | b'A'..=b'Z' | b'_' => return self.make_identifier(),

                _ => return Err(self.error("Unexpected character")),
            })
        } else {
            Ok(self.make_token(TokenKind::Eof))
        }
    }

    fn make_string(&mut self) -> Result<Token> {
        loop {
            match self.peek() {
                Some(b'"') => {
                    self.advance();
                    break;
                }
                Some(b'\n') => {
                    self.line += 1;
                }
                None => {
                    return Err(self.error("Unterminated string."));
                }

                _ => {}
            }
            self.advance();
        }
        Ok(self.make_token(TokenKind::String(
            &self.source[self.start + 1..self.current],
        )))
    }

    fn make_number(&mut self) -> Result<Token> {
        while let Some(b'0'..=b'9') = self.peek() {
            self.advance();
        }

        if let Some(b'.') = self.peek() {
            self.advance();
            while let Some(b'0'..=b'9') = self.peek() {
                self.advance();
            }
        }

        Ok(self.make_token(TokenKind::Number(
            self.source[self.start..self.current].parse().unwrap(),
        )))
    }

    fn make_identifier(&mut self) -> Result<Token> {
        unimplemented!()
    }

    fn advance(&mut self) -> Option<u8> {
        self.current += 1;
        self.byte_iter.next().copied()
    }

    fn peek(&self) -> Option<u8> {
        self.byte_iter.peek().copied().copied()
    }

    fn match_next(&mut self, expected: u8) -> bool {
        if self.peek() == Some(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn make_token(&self, kind: TokenKind) -> Token {
        Token::new(kind, self.line)
    }

    fn skip_whitespace(&mut self) -> Result<()> {
        loop {
            match self.peek() {
                Some(b' ') | Some(b'\r') | Some(b'\t') => {
                    self.advance();
                }

                Some(b'\n') => {
                    self.line += 1;
                    self.advance();
                }

                Some(b'/') => {
                    self.advance();
                    if self.match_next(b'/') {
                        while self.peek().unwrap_or(false) != b'\n' {
                            self.advance();
                        }
                    } else {
                        return self.error("Unexpected character");
                    }
                }
                _ => {
                    break;
                }
            }
        }
        Ok(())
    }

    fn error(&self, msg: &str) -> Error {
        unimplemented!()
    }
}
