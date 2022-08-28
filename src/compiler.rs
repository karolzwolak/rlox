use std::mem;

use crate::{
    bytecode::{self, Precedence},
    scanner::Scanner,
    token::{self, Token, TokenKind},
    Error, Result,
};

pub struct Compiler<'a> {
    scanner: Scanner<'a>,
    error_count: usize,
    current: Option<Token<'a>>,
    previous: Option<Token<'a>>,
    chunk: bytecode::Chunk,
}

impl<'a> Compiler<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            scanner: Scanner::new(source),
            current: None,
            previous: None,
            error_count: 0,
            chunk: bytecode::Chunk::new(),
        }
    }
    
    fn current(&self) -> &Token<'a>{
        self.current.as_ref().unwrap()
    }

    fn previous(&self) -> &Token<'a>{
        self.previous.as_ref().unwrap()
    }

    fn write_ins(&mut self, ins: bytecode::OpCode) {
        self.chunk.write_ins(ins, self.previous().line());
    }

    fn write_constant(&mut self, value: f64) {
        self.chunk
            .add_const_ins(value, self.previous().line());
    }

    pub fn compile(mut self) -> Result<bytecode::Chunk> {
        loop {
            self.advance();
            let token = self.current();
            #[cfg(feature = "trace")]
            self._trace();

            if let token::TokenKind::Eof = token.kind() {
                break;
            }
        }
        if self.error_count != 0 {
            return Err(Error::from(format!(
                "Aborting compilation due to {} errors",
                self.error_count
            )));
        }
        Ok(self.chunk)
    }

    fn synchronize(&mut self) {
        while !self.is_at_end() {
            match self.current.as_ref().unwrap().kind() {
                TokenKind::Semicolon => { 
                    self.advance();
                    return;
                }
                TokenKind::Class
                | TokenKind::Fun
                | TokenKind::Var
                | TokenKind::For
                | TokenKind::If
                | TokenKind::While
                | TokenKind::Print
                | TokenKind::Return => return,
                _ => self.advance(),
            }
        }
    }

    fn is_at_end(&self) -> bool {
        *self.current().kind() == TokenKind::Eof
    }

    fn advance(&mut self) {
        mem::swap(&mut self.previous, &mut self.current);

        let token = self.scanner.scan_token();
        if let Err(error) = token {
            self.report_error(error);
            loop {
                if let Ok(_token) = self.scanner.scan_token() {
                    match _token.kind() {
                        TokenKind::Semicolon |
                        TokenKind::Class
                        | TokenKind::Fun
                        | TokenKind::Var
                        | TokenKind::For
                        | TokenKind::If
                        | TokenKind::While
                        | TokenKind::Print
                        | TokenKind::Return => {
                            self.current = Some(_token);
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }else{
            self.current = Some(token.unwrap());
        }
    }

    fn report_error(&mut self, error: Error) {
        self.error_count += 1;
        eprintln!("{}", error);
    }

    fn error_at(&self, token: &Token<'a>, msg: &str) -> Error{
        let start = token.start();
        Error::from(format!(
            "error: {} at line {}, column {}-{}",
            msg,
            token.line(),
            start,
            start + token.len()
        ))
    }

    fn error_at_previous(&mut self, msg: &str) -> Error {
        self.error_at(self.previous(), msg)
    }

    fn error_at_current(&mut self, msg: &str) -> Error {
        self.error_at(self.current(), msg)
    }

    fn consume(&mut self, expected: TokenKind, msg: &str) -> Result<()> {
        if *self.current().kind() == expected {
            self.advance();
            Ok(())
        } else {
            Err(self.error_at_current(msg))
        }
    }

    fn expression(&mut self) -> Result<()> {
        self.parse_precedence(Precedence::Assignment)
    }

    fn number(&mut self) {
        if let &TokenKind::Number(val, _) = self.previous().kind() {
            self.write_constant(val);
        } else {
            unimplemented!()
        }
    }

    fn grouping(&mut self) -> Result<()> {
        self.expression()?;
        self.consume(TokenKind::RightParen, "Expect ')' after expression")
    }

    fn unary(&mut self) -> Result<()> {
        self.expression()?;
        self.write_ins(bytecode::OpCode::Negate);
        Ok(())
    }

    // parse any expression at given precendece level or higher
    fn parse_precedence(&mut self, predecende: Precedence) -> Result<()> {
        todo!()
    }

    fn binary(&mut self) -> Result<()> {
        let operator = self.previous().kind();
        let precedence = operator.precedence();
        self.parse_precedence(precedence.higher())?;
        Ok(())
    }

    fn _trace(&mut self, token: &token::Token, prev_line: usize) {
        if token.line() != prev_line {
            println!("{:04}", token.line());
        } else {
            print!("   | ");
        }
        print!("{:?} ", token.kind());
    }
}
