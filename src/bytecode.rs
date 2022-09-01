use std::fmt;

#[derive(Debug, Clone, Copy)]
pub enum OpCode {
    Constant(u16),
    Return,
    Negate,
    Not,
    Add,
    Subtract,
    Multiply,
    Divide,

    True,
    False,
    Nil,
}
pub struct Chunk {
    code: Vec<OpCode>,
    constants: Vec<Value>,
    lines: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Value {
    Number(f64),
    String(String),
    Boolean(bool),
    Nil,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(n) => writeln!(f, "Number({})", n),
            Value::String(s) => writeln!(f, "String({})", s),
            Value::Boolean(b) => writeln!(f, "Boolean({})", b),
            Value::Nil => writeln!(f, "Nil"),
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
            OpCode::Negate => "OP_NEGATE".to_string(),
            OpCode::Not => "OP_NOT".to_string(),
            OpCode::Add => "OP_ADD".to_string(),
            OpCode::Subtract => "OP_SUBTRACT".to_string(),
            OpCode::Multiply => "OP_MULTIPLY".to_string(),
            OpCode::Divide => "OP_DIVIDE".to_string(),

            OpCode::True => "OP_TRUE".to_string(),
            OpCode::False => "OP_FALSE".to_string(),
            OpCode::Nil => "OP_NIL".to_string(),
        }
    }
}
