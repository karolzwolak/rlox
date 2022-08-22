use crate::{Result, token};
pub struct Scanner<'a> {
    source: &'a str,
    start: usize,
    current: usize,
    line: usize,
}

impl<'a> Scanner<'a>{
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_token(&mut self) -> Result<token::Token>{
       unimplemented!() 
    }    
    
    fn make_token(&self, kind: token::TokenKind) -> token::Token{
        token::Token::new(kind, self.line)
    }
}