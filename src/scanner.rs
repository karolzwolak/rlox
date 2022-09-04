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
            byte_iter: source.as_bytes().iter().peekable(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    fn advance(&mut self) -> Option<u8> {
        self.current += 1;
        self.byte_iter.next().copied()
    }

    fn peek(&mut self) -> Option<u8> {
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

    fn make_token(&self, kind: TokenKind<'a>) -> Token<'a> {
        Token::new(kind, self.line, self.start)
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
                        while self.peek() != Some(b'\n') {
                            self.advance();
                        }
                    } else {
                        return Err(self.error("Unexpected character"));
                    }
                }
                _ => {
                    break;
                }
            }
        }
        Ok(())
    }

    fn make_token_match(
        &mut self,
        to_match: u8,
        default: TokenKind<'a>,
        matched: TokenKind<'a>,
    ) -> Token<'a> {
        let kind = if self.match_next(to_match) {
            matched
        } else {
            default
        };
        self.make_token(kind)
    }

    fn error(&self, msg: &str) -> Error {
        Error::from(format!(
            "error: {} at line {}, column {}-{} ('{}')",
            msg, self.line, self.start, self.current - 1, &self.source[self.start..self.current] 
        ))
    }

    pub fn scan_token(&mut self) -> Result<Token<'a>> {
        self.skip_whitespace()?;
        self.start = self.current;

        if let Some(ch) = self.advance() {
            Ok(match ch {
                b'(' => self.make_token(TokenKind::LeftParen),
                b')' => self.make_token(TokenKind::RightParen),
                b'{' => self.make_token(TokenKind::LeftBrace),
                b'}' => self.make_token(TokenKind::RightBrace),
                b',' => self.make_token(TokenKind::Comma),
                b'.' => self.make_token(TokenKind::Dot),
                b'-' => self.make_token(TokenKind::Minus),
                b'+' => self.make_token(TokenKind::Plus),
                b';' => self.make_token(TokenKind::Semicolon),
                b'*' => self.make_token(TokenKind::Star),

                b'!' => self.make_token_match(b'=', TokenKind::Bang, TokenKind::BangEqual),
                b'=' => self.make_token_match(b'=', TokenKind::Equal, TokenKind::EqualEqual),
                b'<' => self.make_token_match(b'=', TokenKind::Less, TokenKind::LessEqual),
                b'>' => self.make_token_match(b'=', TokenKind::Greater, TokenKind::GreaterEqual),

                b'"' => return self.make_string(),

                b'0'..=b'9' => return self.make_number(),

                b'a'..=b'z' | b'A'..=b'Z' | b'_' => self.make_identifier(),

                _ => return Err(self.error("Unexpected character")),
            })
        } else {
            Ok(self.make_token(TokenKind::Eof))
        }
    }

    fn make_string(&mut self) -> Result<Token<'a>> {
        loop {
            match self.peek() {
                Some(b'"') => {
                    self.advance();
                    break;
                }
                Some(b'\n') => {
                    self.line += 1;
                }
                Some(b'\0') | None => {
                    return Err(self.error("Unterminated string."));
                }

                _ => {}
            }
            self.advance();
        }
        Ok(self.make_token(TokenKind::String(
            &self.source[self.start + 1..self.current - 1],
        )))
    }

    fn make_number(&mut self) -> Result<Token<'a>> {
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

    fn make_identifier(&mut self) -> Token<'a> {
        let bytes = self.source.as_bytes();
        while let Some(b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_') = self.peek() {
            self.advance();
        }
        self.make_token(match bytes[self.start] {
            b'a' => self.check_keyword(1, "nd", TokenKind::And),
            b'c' => self.check_keyword(1, "lass", TokenKind::Class),
            b'e' => self.check_keyword(1, "lse", TokenKind::Else),
            b'f' => {
                if self.current - self.start > 1 {
                    match bytes[self.start + 1] {
                        b'a' => self.check_keyword(2, "lse", TokenKind::False),
                        b'o' => self.check_keyword(2, "r", TokenKind::For),
                        b'u' => self.check_keyword(2, "n", TokenKind::Fun),
                        _ => self.get_identifier(),
                    }
                } else {
                    self.get_identifier()
                }
            }
            b'i' => self.check_keyword(1, "f", TokenKind::If),
            b'n' => self.check_keyword(1, "il", TokenKind::Nil),
            b'o' => self.check_keyword(1, "r", TokenKind::Or),
            b'p' => self.check_keyword(1, "rint", TokenKind::Print),
            b'r' => self.check_keyword(1, "eturn", TokenKind::Return),
            b's' => self.check_keyword(1, "uper", TokenKind::Super),
            b't' => {
                if self.current - self.start > 1 {
                    match bytes[self.start + 1] {
                        b'h' => self.check_keyword(2, "is", TokenKind::This),
                        b'r' => self.check_keyword(2, "ue", TokenKind::True),
                        _ => self.get_identifier(),
                    }
                } else {
                    self.get_identifier()
                }
            }
            b'v' => self.check_keyword(1, "ar", TokenKind::Var),
            b'w' => self.check_keyword(1, "hile", TokenKind::While),
            _ => self.get_identifier(),
        })
    }

    fn get_identifier(&self) -> TokenKind<'a> {
        TokenKind::Identifier(&self.source[self.start..self.current])
    }

    fn check_keyword(&self, start: usize, rest: &str, kind: TokenKind<'a>) -> TokenKind<'a> {
        let lexeme_start = self.start + start;
        if self.current - self.start == start + rest.len()
            && &self.source[lexeme_start..lexeme_start + rest.len()] == rest
        {
            kind
        } else {
            self.get_identifier()
        }
    }
}
