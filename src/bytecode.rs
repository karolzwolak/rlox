use std::{fmt, rc::Rc};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
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

    JumpIfFalse(Option<u16>),
    Jump(Option<u16>),

    Loop(u16),

    Call(u8),

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
#[derive(Debug, Clone)]
pub struct Chunk {
    code: Vec<OpCode>,
    constants: Vec<Value>,
    lines: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct FunctionObj {
    name: String,
    arity: u8,
    chunk: Chunk,
}

pub enum FunctionKind {
    Function,
    Method,
}

impl fmt::Display for FunctionObj {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<fn {}({})>", self.name, self.arity)
    }
}

impl FunctionObj {
    const MAIN_FUNC_NAME: &'static str = "<Main>";
    pub fn new(name: String, arity: u8) -> Self {
        Self {
            name,
            arity,
            chunk: Chunk::new(),
        }
    }

    pub fn with_chunk(name: String, arity: u8, chunk: Chunk) -> Self {
        Self { name, arity, chunk }
    }

    pub fn new_main() -> Self {
        Self {
            name: Self::MAIN_FUNC_NAME.to_string(),
            arity: 0,
            chunk: Chunk::new(),
        }
    }

    pub fn is_main(&self) -> bool {
        self.name == Self::MAIN_FUNC_NAME
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn arity(&self) -> u8 {
        self.arity
    }

    pub fn arity_mut(&mut self) -> &mut u8 {
        &mut self.arity
    }

    pub fn chunk(&self) -> &Chunk {
        &self.chunk
    }

    pub fn chunk_mut(&mut self) -> &mut Chunk {
        &mut self.chunk
    }

    pub fn disassemble(&self) {
        self.chunk.disassemble(&self.name);
    }
}

#[derive(Debug)]
pub enum Value {
    Number(f64),
    String(Rc<String>),
    Function(usize),
    Boolean(bool),
    Nil,
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Nil => false,
            Value::Boolean(b) => *b,
            _ => true,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Nil => write!(f, "<Nil>"),
            Value::Function(id) => write!(f, "#{}", id),
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
            (Value::Function(a), Value::Function(b)) => a == b,
            _ => false,
        }
    }
}

impl Clone for Value {
    fn clone(&self) -> Self {
        match self {
            Self::Number(n) => Self::Number(*n),
            Self::String(s) => Self::String(Rc::clone(s)),
            Self::Function(id) => Self::Function(*id),
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

    pub fn get_line(&self, index: usize) -> usize {
        self.lines[index]
    }

    pub fn len(&self) -> usize {
        self.code.len()
    }

    pub fn code(&self) -> &[OpCode] {
        &self.code
    }

    pub fn code_mut(&mut self) -> &mut Vec<OpCode> {
        &mut self.code
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
                format!("OP_CONSTANT<#{:04}, '{}'>", index, chunk.get_const(*index))
            }
            OpCode::Return => "OP_RETURN".to_string(),
            OpCode::Print => "OP_PRINT".to_string(),
            OpCode::Pop => "OP_POP".to_string(),

            OpCode::DefineGlobal(index) => format!("OP_DEFINE_GLOBAL<#{:04}>", index),
            OpCode::GetGlobal(index) => format!("OP_GET_GLOBAL<#{:04}>", index),
            OpCode::SetGlobal(index) => format!("OP_SET_GLOBAL<#{:04}>", index),

            OpCode::GetLocal(index) => format!("OP_GET_LOCAL<s#{:04}>", index),
            OpCode::SetLocal(index) => format!("OP_SET_LOCAL<s#{:04}>", index),

            OpCode::JumpIfFalse(offset) => format!("OP_JUMP_IF_FALSE<+{:04}>", offset.unwrap_or(0)),
            OpCode::Jump(offset) => format!("OP_JUMP<{:+04}>", offset.unwrap_or(0)),

            OpCode::Loop(offset) => format!("OP_LOOP<-{:04}>", offset),

            OpCode::Call(arg_count) => format!("OP_CALL<{}>", arg_count),

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
