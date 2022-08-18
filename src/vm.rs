use crate::bytecode;
use std::io::Cursor;
pub struct VM<'a> {
    chunk: &'a bytecode::Chunk,
    ip: usize,
    mode: ExecutionMode,
}

pub enum ExecutionMode {
    Normal,
    Debug,
}

pub enum InterpretResult {
    Ok,
    CompileError,
    RuntimeError,
}

impl<'a> VM<'a> {
    pub fn with_chunk(chunk: &'a bytecode::Chunk, mode: ExecutionMode) -> Self {
        Self { chunk, ip: 0 
        , mode }
    }

    fn trace_ins(&self){
       println!("ins: {}", self.chunk.dissassemble_ins(self.ip - 1)); 
    }

    pub fn interpret(&mut self) -> InterpretResult {
        let code = self.chunk.code();
        while !self.is_at_end() {
            let ins = self.advance();
            if let ExecutionMode::Debug = self.mode {
                self.trace_ins();
            }
            match ins{
                bytecode::OpCode::Constant(index) => {
                    let value = self.chunk.get_const(*index);
                    println!("{}", value);
                }
                bytecode::OpCode::Return => break,
            }
        }
        InterpretResult::Ok
    }

    fn is_at_end(&self) -> bool {
        self.ip >= self.chunk.code().len()
    }

    fn advance(&mut self) -> &'a bytecode::OpCode {
        let ins = &self.chunk.code()[self.ip];
        self.ip += 1;
        ins
    }
}
