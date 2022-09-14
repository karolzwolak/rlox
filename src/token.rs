use core::fmt;

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

            TokenKind::And => Precedence::And,

            TokenKind::Or => Precedence::Or,

            _ => Precedence::None,
        }
    }
}

impl<'a> fmt::Display for TokenKind<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s;
        write!(
            f,
            "{}",
            match self {
                TokenKind::Eof => "<EOF>",
                TokenKind::None => "<NONE>",
                TokenKind::LeftParen => "(",
                TokenKind::RightParen => ")",
                TokenKind::LeftBrace => "{",
                TokenKind::RightBrace => "}",
                TokenKind::Comma => ",",
                TokenKind::Dot => ".",
                TokenKind::Minus => "-",
                TokenKind::Plus => "+",
                TokenKind::Semicolon => ";",
                TokenKind::Slash => "/",
                TokenKind::Star => "*",
                TokenKind::Bang => "!",
                TokenKind::BangEqual => "!=",
                TokenKind::Equal => "=",
                TokenKind::EqualEqual => "==",
                TokenKind::Greater => ">",
                TokenKind::GreaterEqual => ">=",
                TokenKind::Less => "<",
                TokenKind::LessEqual => "<=",
                TokenKind::Identifier(s) => s,
                TokenKind::String(s) => s,
                TokenKind::Number(n) => {
                    s = n.to_string();
                    &s
                }
                TokenKind::And => "and",
                TokenKind::Class => "class",
                TokenKind::Else => "else",
                TokenKind::False => "false",
                TokenKind::Fun => "fun",
                TokenKind::For => "for",
                TokenKind::If => "if",
                TokenKind::Nil => "nil",
                TokenKind::Or => "or",
                TokenKind::Print => "print",
                TokenKind::Return => "return",
                TokenKind::Super => "super",
                TokenKind::This => "this",
                TokenKind::True => "true",
                TokenKind::Var => "var",
                TokenKind::While => "while",
            }
        )
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
}
