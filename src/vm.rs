use crate::bytecode::{self, OpCode};
pub struct VM<'a> {
    chunk: &'a bytecode::Chunk,
    ip: usize,
    stack: Vec<bytecode::Value>,
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

    fn _trace(&self) {
        println!("stack: {:?}", self.stack);
        println!("ins:   {}", self.chunk.dissassemble_ins(self.ip));
    }

    fn get_binary_operands(&mut self) -> (bytecode::Value, bytecode::Value) {
        let b = self.stack.pop().unwrap();
        let a = self.stack.pop().unwrap();
        (a, b)
    }

    pub fn run(&mut self) -> crate::Result<()> {
        while !self.is_at_end() {
            #[cfg(feature = "trace")]
            self._trace();

            let ins = self.advance();
            match ins {
                OpCode::Constant(index) => {
                    let value = self.chunk.get_const(*index);
                    self.stack.push(value);
                }
                OpCode::Negate => {
                    let value = self.stack.pop().unwrap();
                    self.stack.push(-value);
                }
                OpCode::Add => {
                    let (a, b) = self.get_binary_operands();
                    self.stack.push(a + b);
                }
                OpCode::Subtract => {
                    let (a, b) = self.get_binary_operands();
                    self.stack.push(a - b);
                }
                OpCode::Multiply => {
                    let (a, b) = self.get_binary_operands();
                    self.stack.push(a * b);
                }
                OpCode::Divide => {
                    let (a, b) = self.get_binary_operands();
                    self.stack.push(a / b);
                }
                OpCode::Return => {
                    println!("{}", self.stack.pop().unwrap());
                    break;
                }
            }
        }
        Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bytecode::{Chunk, OpCode};

    #[test]
    fn test_ok_interpret() {
        let mut chunk = Chunk::new();
        chunk.add_const_ins(1.2, 123);
        chunk.add_const_ins(3.4, 123);
        chunk.write_ins(OpCode::Add, 123);
        chunk.write_ins(OpCode::Return, 123);

        let mut vm = VM::with_chunk(&chunk);
        assert!(matches!(vm.run(), Ok(())));
    }
    #[test]
    fn test_binary_ops() {
        let mut chunk = Chunk::new();

        chunk.add_const_ins(5., 123);
        chunk.add_const_ins(3., 123);
        chunk.write_ins(OpCode::Add, 123);
        chunk.add_const_ins(4., 123);
        chunk.write_ins(OpCode::Divide, 123);
        chunk.write_ins(OpCode::Negate, 123);
        chunk.add_const_ins(2.5, 123);
        chunk.write_ins(OpCode::Multiply, 123);

        let mut vm = VM::with_chunk(&chunk);
        assert!(matches!(vm.run(), Ok(())));
        assert_eq!(vm.stack, vec![-5.]);
    }
}
