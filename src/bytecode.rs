use std::fmt;
pub enum OpCode{
    Constant(usize),
    Return,
}
pub struct Chunk{
    code: Vec<OpCode>,
    constants: Vec<Value>,
    lines: Vec<usize>,
}

pub type Value = f64;

impl Chunk{
    pub fn new() -> Chunk{
        Chunk{
            code: Vec::new(),
            constants: Vec::new(),
            lines: Vec::new(),
        }
    }
    pub fn write_chunk(&mut self, byte: OpCode, line: usize){
        self.lines.push(line);
        self.code.push(byte);
    }
    pub fn add_const(&mut self, value: Value) -> usize{
        self.constants.push(value);
        self.constants.len() - 1
    }
    pub fn get_const(&self, index: usize) -> Value{
        self.constants[index]
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Chunk{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (offset, instruction) in self.code.iter().enumerate(){
            if offset > 0 && self.lines[offset] == self.lines[offset - 1]{
                write!(f, "   | ")?;
            }else{
                write!(f, "{:04} ", self.lines[offset])?;
            }
            writeln!(f, "#{:04} {}", offset, instruction.dissassemble(self))?;
        }
        Ok(())
    }
}

impl OpCode{
    pub fn dissassemble(&self, chunk: &Chunk) -> String{
        match self{
            OpCode::Constant(index) => format!("OP_CONSTANT #{:04} '{}'", index, chunk.get_const(*index)),
            OpCode::Return => "OP_RETURN".to_string(),
        }
    }
}

