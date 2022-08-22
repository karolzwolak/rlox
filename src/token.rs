
#[derive(Debug)]
pub enum TokenKind<'a> {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Identifier(&'a str),
    String(&'a str),
    Number(f64),

    // Keywords.
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Eof,
}


pub struct Token<'a>{
    kind: TokenKind<'a>,
    line: usize,
}

impl<'a> Token<'a> {
    pub fn new(kind: TokenKind<'a>, line: usize) -> Self {
        Self {
            kind,
            line,
        }
    }
    pub fn kind(&self) -> &TokenKind<'a>{
        &self.kind
    }
    pub fn line(&self) -> usize{
        self.line
    }
}