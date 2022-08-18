use std::fmt;

#[derive(Debug, Clone, Copy)]
pub enum OpCode{
    Constant(u16),
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
    pub fn add_const(&mut self, value: Value) -> u16{
        self.constants.push(value);
        self.constants.len() as u16 - 1
    }
    pub fn get_const(&self, index: u16) -> Value{
        self.constants[index as usize]
    }

    pub fn code(&self) -> &[OpCode]{
        &self.code
    }

    pub fn dissassemble_ins(&self, offset: usize) -> String{
        let prefix = 
            if offset > 0 && self.lines[offset] == self.lines[offset - 1]{
                "   |".to_string()
            }else{
                format!("{:04}", self.lines[offset])
            };
            format!("l{prefix}  #{:04} {}", offset, self.code[offset].dissassemble(self))
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Chunk{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (offset, _) in self.code.iter().enumerate(){
            writeln!(f, "{}", self.dissassemble_ins(offset))?;
        }
        Ok(())
    }
}

impl OpCode{
    pub fn dissassemble(&self, chunk: &Chunk) -> String{
        match self{
            OpCode::Constant(index) => format!("OP_CONSTANT<#{:04}, '{}'>", index, chunk.get_const(*index)),
            OpCode::Return => "OP_RETURN".to_string(),
        }
    }
}

