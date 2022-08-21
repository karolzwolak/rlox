use crate::bytecode;
pub struct VM<'a> {
    chunk: &'a bytecode::Chunk,
    ip: usize,
    stack: Vec<bytecode::Value>,
}

pub enum InterpretResult {
    Ok,
    CompileError,
    RuntimeError,
}

impl<'a> VM<'a> {
    const STACK_MAX: usize = 256;
    pub fn with_chunk(chunk: &'a bytecode::Chunk) -> Self {
        Self {
            chunk,
            ip: 0,
            stack: Vec::with_capacity(Self::STACK_MAX),
        }
    }

    fn _trace_ins(&self) {
        println!("ins:   {}", self.chunk.dissassemble_ins(self.ip));
    }

    pub fn interpret(&mut self) -> InterpretResult {
        let code = self.chunk.code();
        while !self.is_at_end() {
            #[cfg(feature = "trace")]
            {
                println!("stack: {:?}", self.stack);
                self._trace_ins();
            }
            let ins = self.advance();
            match ins {
                bytecode::OpCode::Constant(index) => {
                    let value = self.chunk.get_const(*index);
                    self.stack.push(value);
                }
                bytecode::OpCode::Return => {
                    println!("{}", self.stack.pop().unwrap());
                    break;
                }
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
