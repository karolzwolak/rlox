use std::mem;

use crate::{
    bytecode::{self, Precedence, OpCode, Value},
    scanner::Scanner,
    token::{self, Token, TokenKind},
    Error, Result,
};

pub struct Compiler<'a> {
    scanner: Scanner<'a>,
    error_count: usize,
    current: Token<'a>,
    previous: Token<'a>,
    chunk: bytecode::Chunk,
}

impl<'a> Compiler<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            scanner: Scanner::new(source),
            current: Token::none(),
            previous: Token::none(),
            error_count: 0,
            chunk: bytecode::Chunk::new(),
        }
    }
    fn write_ins(&mut self, ins: bytecode::OpCode) {
        self.chunk.write_ins(ins, self.previous.line());
    }

    fn write_constant(&mut self, value: Value) {
        self.chunk.add_const_ins(value, self.previous.line());
    }

    pub fn compile(mut self) -> Result<bytecode::Chunk> {
        self.advance()?;
        self.expression()?;
        self.write_ins(OpCode::Return);
        self.consume(TokenKind::Eof, "Expect end of expression.")?;
        if self.error_count != 0 {
            return Err(Error::from(format!(
                "Aborting compilation due to {} errors",
                self.error_count
            )));
        }

        #[cfg(feature = "print_code")]
        self.chunk.disassemble("code");

        Ok(self.chunk)
    }

    fn synchronize(&mut self) {
        let curr = loop {
            if let Ok(token) = self.scanner.scan_token() {
                match token.kind() {
                    TokenKind::Semicolon => match self.scanner.scan_token() {
                        Ok(next) => {
                            self.previous = token;
                            break next;
                        }

                        Err(error) => {
                            self.report_error(error);
                            continue;
                        }
                    },
                    TokenKind::Eof
                    | TokenKind::Class
                    | TokenKind::Fun
                    | TokenKind::Var
                    | TokenKind::For
                    | TokenKind::If
                    | TokenKind::While
                    | TokenKind::Print
                    | TokenKind::Return => break token,
                    _ => {}
                }
            }
        };

        self.previous = mem::replace(&mut self.current, curr);
    }

    fn is_at_end(&self) -> bool {
        *self.current.kind() == TokenKind::Eof
    }

    fn advance(&mut self) -> Result<()> {
        match self.scanner.scan_token() {
            Ok(token) => {
                self.previous = mem::replace(&mut self.current, token);
                Ok(())
            }
            Err(error) => Err(error),
        }
    }

    fn report_error(&mut self, error: Error) {
        self.error_count += 1;
        eprintln!("{}", error);
    }

    fn error_at(&self, token: &Token<'a>, msg: &str) -> Error {
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
        self.error_at(&self.previous, msg)
    }

    fn error_at_current(&mut self, msg: &str) -> Error {
        self.error_at(&self.current, msg)
    }

    fn consume(&mut self, expected: TokenKind, msg: &str) -> Result<()> {
        if *self.current.kind() == expected {
            self.advance()?;
            Ok(())
        } else {
            Err(self.error_at_current(msg))
        }
    }

    fn expression(&mut self) -> Result<()> {
        self.parse_precedence(Precedence::Assignment)
    }

    fn number(&mut self) {
        if let &TokenKind::Number(val, _) = self.previous.kind() {
            self.write_constant(Value::Number(val));
        } else {
            unreachable!();
        }
    }

    fn grouping(&mut self) -> Result<()> {
        self.expression()?;
        self.consume(TokenKind::RightParen, "Expect ')' after expression")
    }

    fn unary(&mut self) -> Result<()> {
        self.parse_precedence(Precedence::Unary)?;
        self.write_ins(bytecode::OpCode::Negate);
        Ok(())
    }

    // parse any expression at given precendece level or higher
    fn parse_precedence(&mut self, precedence: Precedence) -> Result<()> {
        self.advance()?;
        self.prefix(*self.previous.kind())?;

        while precedence <= self.current.kind().precedence() {
            self.advance()?;
            self.infix(*self.previous.kind())?;
        }
        Ok(())
    }

    fn binary(&mut self) -> Result<()> {
        let operator = *self.previous.kind();
        let precedence = operator.precedence();
        self.parse_precedence(precedence.higher())?;

        match operator{
            TokenKind::Plus => self.write_ins(bytecode::OpCode::Add),
            TokenKind::Minus => self.write_ins(bytecode::OpCode::Subtract),
            TokenKind::Star => self.write_ins(bytecode::OpCode::Multiply),
            TokenKind::Slash => self.write_ins(bytecode::OpCode::Divide),
            _ => todo!(),
        }
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

    fn prefix(&mut self, kind: TokenKind) -> Result<()> {
        match kind {
            TokenKind::LeftParen => self.grouping(),
            TokenKind::Number(..) => {
                self.number();
                Ok(())
            }
            TokenKind::Minus => self.unary(),

            _ => unimplemented!("Unimplemented prefix for {:?}", kind),
        }
    }

    fn infix(&mut self, kind: TokenKind) -> Result<()> {
        match kind {
            TokenKind::Minus | TokenKind::Plus | TokenKind::Slash | TokenKind::Star => {
                self.binary()
            }

            _ => Err(self.error_at_previous("Expected expression")),
        }
    }
}
