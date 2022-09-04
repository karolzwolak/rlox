use std::{mem, rc::Rc};

use crate::{
    bytecode::{self, OpCode, Precedence, Value},
    scanner::Scanner,
    token::{self, Token, TokenKind},
    Error, Result,
};

/*
statement      → exprStmt
               | forStmt
               | ifStmt
               | printStmt
               | returnStmt
               | whileStmt
               | block ;

declaration    → classDecl
               | funDecl
               | varDecl
               | statement ;

*/

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

    fn add_const(&mut self, val: Value) -> u16 {
        self.chunk.add_const(val)
    }

    fn write_const_ins(&mut self, value: Value) {
        self.chunk.add_const_ins(value, self.previous.line())
    }

    pub fn compile(mut self) -> Result<bytecode::Chunk> {
        self.advance()?;
        while !self.is_at_end() {
            self.declaration();
        }
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

    fn declaration(&mut self) {
        let result = match self.current.kind() {
            TokenKind::Var => self.var_decl(),
            _ => self.statement(),
        };
        if let Err(error) = result {
            self.synchronize();
            self.report_error(error);
        }
    }

    fn statement(&mut self) -> Result<()> {
        match self.current.kind() {
            TokenKind::Print => self.print_stmt(),
            _ => self.expression_stmt(),
        }
    }

    fn write_ident_constant(&mut self, ident: &'a str) -> u16 {
        let ident = Value::String(Rc::new(ident.to_string()));
        self.add_const(ident)
    }

    fn var_decl(&mut self) -> Result<()> {
        self.advance()?;
        let name = self.consume_ident("Expect variable name.")?;
        let id = self.write_ident_constant(name);
        if let Ok(true) = self.match_curr(&TokenKind::Equal) {
            self.expression()?;
        } else {
            self.write_ins(OpCode::Nil);
        }

        self.consume(
            TokenKind::Semicolon,
            "Expect ';' after variable declaration.",
        )?;
        self.write_ins(OpCode::DefineGlobal(id));
        Ok(())
    }

    fn variable(&mut self, ident: &'a str, can_assign: bool) -> Result<()> {
        let id = self.write_ident_constant(ident);
        if can_assign && self.match_curr(&TokenKind::Equal)? {
            self.expression()?;
            self.write_ins(OpCode::SetGlobal(id));
        } else {
            self.write_ins(OpCode::GetGlobal(id));
        }
        Ok(())
    }

    fn print_stmt(&mut self) -> Result<()> {
        self.advance()?;
        self.expression()?;
        self.consume(TokenKind::Semicolon, "Expect ';' after value.")?;
        self.write_ins(OpCode::Print);
        Ok(())
    }

    fn expression_stmt(&mut self) -> Result<()> {
        self.expression()?;
        self.consume(TokenKind::Semicolon, "Expect ';' after expression.")?;
        self.write_ins(OpCode::Pop);
        Ok(())
    }

    fn is_at_end(&self) -> bool {
        self.check_curr(&TokenKind::Eof)
    }

    fn match_curr(&mut self, kind: &TokenKind) -> Result<bool> {
        if self.check_curr(kind) {
            self.advance()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn check_curr(&self, kind: &TokenKind) -> bool {
        *self.current.kind() == *kind
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
        Error::from(format!(
            "{} at line {}, at token '{}'",
            msg,
            token.line(),
            token.kind()
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

    fn consume_ident(&mut self, msg: &str) -> Result<&'a str> {
        if let TokenKind::Identifier(ident) = *self.current.kind() {
            self.advance()?;
            Ok(ident)
        } else {
            Err(self.error_at_current(msg))
        }
    }

    fn expression(&mut self) -> Result<()> {
        self.parse_precedence(Precedence::Assignment)
    }

    fn grouping(&mut self) -> Result<()> {
        self.expression()?;
        self.consume(TokenKind::RightParen, "Expect ')' after expression")
    }

    fn unary(&mut self) -> Result<()> {
        let op = *self.previous.kind();
        self.parse_precedence(Precedence::Unary)?;
        self.write_ins(match op {
            TokenKind::Bang => OpCode::Not,
            TokenKind::Minus => OpCode::Negate,
            _ => unreachable!(),
        });
        Ok(())
    }

    // parse any expression at given precendece level or higher
    fn parse_precedence(&mut self, precedence: Precedence) -> Result<()> {
        self.advance()?;
        let can_assign = precedence <= Precedence::Assignment;
        self.prefix(*self.previous.kind(), can_assign)?;

        while precedence <= self.current.kind().precedence() {
            self.advance()?;
            self.infix(*self.previous.kind())?;
        }
        if can_assign && self.match_curr(&TokenKind::Equal)? {
            return Err(self.error_at_previous("Invalid assignment target."));
        }
        Ok(())
    }

    fn binary(&mut self) -> Result<()> {
        let operator = *self.previous.kind();
        let precedence = operator.precedence();
        self.parse_precedence(precedence.higher())?;

        match operator {
            TokenKind::Plus => self.write_ins(OpCode::Add),
            TokenKind::Minus => self.write_ins(OpCode::Subtract),
            TokenKind::Star => self.write_ins(OpCode::Multiply),
            TokenKind::Slash => self.write_ins(OpCode::Divide),
            TokenKind::BangEqual => {
                self.write_ins(OpCode::Equal);
                self.write_ins(OpCode::Not);
            }
            TokenKind::EqualEqual => self.write_ins(OpCode::Equal),
            TokenKind::Less => self.write_ins(OpCode::Less),
            TokenKind::LessEqual => {
                self.write_ins(OpCode::Greater);
                self.write_ins(OpCode::Not);
            }

            TokenKind::Greater => self.write_ins(OpCode::Greater),
            TokenKind::GreaterEqual => {
                self.write_ins(OpCode::Less);
                self.write_ins(OpCode::Not);
            }

            _ => unreachable!(),
        }
        Ok(())
    }

    fn literal(&mut self) {
        match self.previous.kind() {
            TokenKind::True => self.write_ins(OpCode::True),
            TokenKind::False => self.write_ins(OpCode::False),
            TokenKind::Nil => self.write_ins(OpCode::Nil),
            _ => unreachable!(),
        }
    }

    fn _trace(&mut self, token: &token::Token, prev_line: usize) {
        if token.line() != prev_line {
            println!("{:04}", token.line());
        } else {
            print!("   | ");
        }
        print!("{:?} ", token.kind());
    }

    fn prefix(&mut self, kind: TokenKind<'a>, can_assign: bool) -> Result<()> {
        match kind {
            TokenKind::LeftParen => self.grouping(),
            TokenKind::Number(val) => {
                self.write_const_ins(Value::Number(val));
                Ok(())
            }
            TokenKind::Minus | TokenKind::Bang => self.unary(),
            TokenKind::True | TokenKind::False | TokenKind::Nil => {
                self.literal();
                Ok(())
            }
            TokenKind::String(s) => {
                self.write_const_ins(Value::String(Rc::new(s.to_string())));
                Ok(())
            }
            TokenKind::Identifier(ident) => self.variable(ident, can_assign),

            _ => Err(self.error_at_previous(&format!("Unexpected token '{:?}'", kind))),
        }
    }

    fn infix(&mut self, kind: TokenKind) -> Result<()> {
        match kind {
            TokenKind::Minus
            | TokenKind::Plus
            | TokenKind::Slash
            | TokenKind::Star
            | TokenKind::EqualEqual
            | TokenKind::Bang
            | TokenKind::Greater
            | TokenKind::GreaterEqual
            | TokenKind::Less
            | TokenKind::LessEqual => self.binary(),

            _ => Ok(()),
        }
    }
}
