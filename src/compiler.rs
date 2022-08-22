use crate::{bytecode, scanner::Scanner, token, Error, Result};

pub struct Compiler<'a> {
    scanner: Scanner<'a>,
    errors: Vec<Error>,
}

impl<'a> Compiler<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            scanner: Scanner::new(source),
            errors: Vec::new(),
        }
    }

    pub fn compile(&mut self, source: &str) -> Result<bytecode::Chunk> {
        let mut scanner = Scanner::new(source);

        loop {
            let token = self.scanner.scan_token();
            if let Err(error) = token {
                self.errors.push(error);
                continue;
            }
            let token = token.unwrap();
            #[cfg(feature = "trace")]
            self._trace();

            if let token::TokenKind::Eof = token.kind() {
                break;
            }
        }
        Ok(bytecode::Chunk::new())
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
