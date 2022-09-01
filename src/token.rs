use crate::bytecode::Precedence;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TokenKind<'a> {
    Eof,
    None,

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
    Number(f64, u8),

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
}
impl<'a> TokenKind<'a> {
    pub fn precedence(&self) -> Precedence {
        match self {
            TokenKind::Slash | TokenKind::Star => Precedence::Factor,
            TokenKind::Bang => Precedence::Unary,
            TokenKind::EqualEqual | TokenKind::BangEqual => Precedence::Equality,
            TokenKind::Greater
            | TokenKind::Less
            | TokenKind::GreaterEqual
            | TokenKind::LessEqual => Precedence::Comparison,
            TokenKind::Plus | TokenKind::Minus => Precedence::Term,
            TokenKind::Equal => Precedence::Assignment,
            _ => Precedence::None,
        }
    }
}

pub struct Token<'a> {
    kind: TokenKind<'a>,
    line: usize,
    start: usize,
}

impl<'a> Token<'a> {
    pub fn new(kind: TokenKind<'a>, line: usize, start: usize) -> Self {
        Self { kind, start, line }
    }

    pub fn none() -> Self {
        Self::new(TokenKind::None, 0, 0)
    }
    pub fn kind(&self) -> &TokenKind<'a> {
        &self.kind
    }
    pub fn line(&self) -> usize {
        self.line
    }
    pub fn start(&self) -> usize {
        self.start
    }

    pub fn len(&self) -> usize {
        match self.kind {
            TokenKind::Identifier(s) | TokenKind::String(s) => s.len(),
            TokenKind::Number(_, len) => len as usize,
            TokenKind::Eof | TokenKind::None => 0,
            TokenKind::LeftParen
            | TokenKind::RightParen
            | TokenKind::LeftBrace
            | TokenKind::RightBrace
            | TokenKind::Comma
            | TokenKind::Dot
            | TokenKind::Minus
            | TokenKind::Plus
            | TokenKind::Semicolon
            | TokenKind::Slash
            | TokenKind::Star
            | TokenKind::Bang
            | TokenKind::Equal
            | TokenKind::Greater
            | TokenKind::Less => 1,
            TokenKind::Or
            | TokenKind::If
            | TokenKind::BangEqual
            | TokenKind::GreaterEqual
            | TokenKind::LessEqual
            | TokenKind::EqualEqual => 2,
            TokenKind::And | TokenKind::For | TokenKind::Nil | TokenKind::Fun | TokenKind::Var => 3,
            TokenKind::True | TokenKind::This | TokenKind::Else => 4,
            TokenKind::False | TokenKind::Super | TokenKind::While | TokenKind::Class => 5,

            TokenKind::Print | TokenKind::Return => 6,
        }
    }
}
