use std::{fmt, rc::Rc};

#[derive(Debug, Clone, Copy)]
pub enum OpCode {
    Constant(u16),
    Return,
    Print,
    Pop,

    DefineGlobal(u16),
    GetGlobal(u16),
    SetGlobal(u16),

    GetLocal(u16),
    SetLocal(u16),

    Negate,
    Not,
    Add,
    Subtract,
    Multiply,
    Divide,

    Less,
    Greater,
    Equal,

    True,
    False,
    Nil,
}
pub struct Chunk {
    code: Vec<OpCode>,
    constants: Vec<Value>,
    lines: Vec<usize>,
}

#[derive(Debug, PartialOrd)]
pub enum Value {
    Number(f64),
    String(Rc<String>),
    Boolean(bool),
    Nil,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Nil => write!(f, "<Nil>"),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Nil, Value::Nil) => true,
            _ => false,
        }
    }
}

impl Clone for Value {
    fn clone(&self) -> Self {
        match self {
            Self::Number(n) => Self::Number(*n),
            Self::String(s) => Self::String(Rc::clone(s)),
            Self::Boolean(b) => Self::Boolean(*b),
            Self::Nil => Self::Nil,
        }
    }
}

#[derive(PartialEq, PartialOrd, Ord, Eq, Clone, Copy)]
// Higher precedence means that it will be evaluated first.
pub enum Precedence {
    None,
    Assignment,
    Or,
    And,
    Equality,
    Comparison,
    Term,
    Factor,
    Unary,
    Call,
    Primary,
}

impl Precedence {
    pub fn higher(&self) -> Self {
        match self {
            Precedence::None => Precedence::Assignment,
            Precedence::Assignment => Precedence::Or,
            Precedence::Or => Precedence::And,
            Precedence::And => Precedence::Equality,
            Precedence::Equality => Precedence::Comparison,
            Precedence::Comparison => Precedence::Term,
            Precedence::Term => Precedence::Factor,
            Precedence::Factor => Precedence::Unary,
            Precedence::Unary => Precedence::Call,
            Precedence::Call | Precedence::Primary => Precedence::Primary,
        }
    }
}

impl Chunk {
    pub fn new() -> Chunk {
        Chunk {
            code: Vec::new(),
            constants: Vec::new(),
            lines: Vec::new(),
        }
    }
    pub fn write_ins(&mut self, byte: OpCode, line: usize) {
        self.lines.push(line);
        self.code.push(byte);
    }
    pub fn add_const(&mut self, value: Value) -> u16 {
        self.constants.push(value);
        (self.constants.len() - 1) as u16
    }
    pub fn add_const_ins(&mut self, value: Value, line: usize) {
        let constant = self.add_const(value);
        self.write_ins(OpCode::Constant(constant), line);
    }
    pub fn get_const(&self, index: u16) -> &Value {
        &self.constants[index as usize]
    }

    pub fn code(&self) -> &[OpCode] {
        &self.code
    }

    pub fn dissassemble_ins(&self, offset: usize) -> String {
        let prefix = if offset > 0 && self.lines[offset] == self.lines[offset - 1] {
            "   |".to_string()
        } else {
            format!("{:04}", self.lines[offset])
        };
        format!(
            "l{prefix}  #{:04} {}",
            offset,
            self.code[offset].dissassemble(self)
        )
    }

    pub fn disassemble(&self, name: &str) {
        println!("trace chunk '{}'\n{}", name, self);
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (offset, _) in self.code.iter().enumerate() {
            writeln!(f, "{}", self.dissassemble_ins(offset))?;
        }
        Ok(())
    }
}

impl OpCode {
    pub fn dissassemble(&self, chunk: &Chunk) -> String {
        match self {
            OpCode::Constant(index) => {
                format!(
                    "OP_CONSTANT<#{:04}, '{:?}'>",
                    index,
                    chunk.get_const(*index)
                )
            }
            OpCode::Return => "OP_RETURN".to_string(),
            OpCode::Print => "OP_PRINT".to_string(),
            OpCode::Pop => "OP_POP".to_string(),

            OpCode::DefineGlobal(index) => format!("OP_DEFINE_GLOBAL<#{:04}>", index),
            OpCode::GetGlobal(index) => format!("OP_GET_GLOBAL<#{:04}>", index),
            OpCode::SetGlobal(index) => format!("OP_SET_GLOBAL<#{:04}>", index),

            OpCode::GetLocal(index) => format!("OP_GET_LOCAL<s#{:04}>", index),
            OpCode::SetLocal(index) => format!("OP_SET_LOCAL<s#{:04}>", index),

            OpCode::Negate => "OP_NEGATE".to_string(),
            OpCode::Not => "OP_NOT".to_string(),
            OpCode::Add => "OP_ADD".to_string(),
            OpCode::Subtract => "OP_SUBTRACT".to_string(),
            OpCode::Multiply => "OP_MULTIPLY".to_string(),
            OpCode::Divide => "OP_DIVIDE".to_string(),

            OpCode::Greater => "OP_GREATER".to_string(),
            OpCode::Less => "OP_LESS".to_string(),
            OpCode::Equal => "OP_EQUAL".to_string(),

            OpCode::True => "OP_TRUE".to_string(),
            OpCode::False => "OP_FALSE".to_string(),
            OpCode::Nil => "OP_NIL".to_string(),
        }
    }
}
