use crate::{
    bytecode::{self, OpCode, Value},
    Error, Result,
};
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

    pub fn run(&mut self) -> crate::Result<()> {
        while !self.is_at_end() {
            #[cfg(feature = "trace")]
            self._trace();

            let ins = self.advance();
            match ins {
                OpCode::Constant(index) => self.add_const(*index),
                OpCode::Negate => self.negate()?,
                OpCode::Add => self.add()?,
                op @ (OpCode::Subtract | OpCode::Multiply | OpCode::Divide) => self.binary(op)?,
                OpCode::Return => {
                    println!("{:?}", self.stack.pop().unwrap());
                    break;
                }
            }
        }
        Ok(())
    }

    fn add_const(&mut self, id: u16) {
        // todo- remove that clone
        self.push_stack(self.chunk.get_const(id).clone());
    }

    fn push_stack(&mut self, v: Value) {
        self.stack.push(v)
    }

    fn negate(&mut self) -> Result<()> {
        match self.pop_stack() {
            Value::Number(n) => self.push_stack(Value::Number(-n)),
            Value::Boolean(b) => self.push_stack(Value::Boolean(!b)),
            v => return Err(self.runtime_error(&format!("Cannot negate {v}"))),
        };
        Ok(())
    }

    fn add(&mut self) -> Result<()> {
        let b = self.pop_stack();
        let a = self.pop_stack();
        match (a, b) {
            (Value::Number(a), Value::Number(b)) => {
                self.stack.push(Value::Number(a + b));
            }
            (Value::String(s1), Value::String(s2)) => {
                self.stack.push(Value::String(format!("{s1}{s2}")));
            }
            (a, b) => return Err(self.runtime_error(&format!("Cannot add {a} and {b}"))),
        };
        Ok(())
    }

    fn pop_stack(&mut self) -> Value {
        if let Some(value) = self.stack.pop() {
            value
        } else {
            self.internal_error("Tried to pop an empty stack")
        }
    }

    fn pop_number(&mut self) -> Result<f64> {
        match self.pop_stack() {
            Value::Number(n) => Ok(n),
            v => Err(self.runtime_error(&format!("Expected a number but got {}", v))),
        }
    }

    fn peek_stack(&self, offset: usize) -> Option<&Value> {
        self.stack.get(self.stack.len() - 1 - offset)
    }

    // fn peek_stack_mut(&mut self, offset: usize) -> Option<&mut Value> {
    //     self.stack.get_mut(self.stack.len() - 1 - offset)
    // }

    fn peek_number(&mut self, offset: usize) -> Result<&f64> {
        match self.peek_stack(offset) {
            None => self.internal_error(&format!(
                "Expected value at stack at index {} but found no value",
                self.stack.len() - 1 - offset
            )),

            Some(Value::Number(n)) => Ok(n),
            Some(v) => Err(self.runtime_error(&format!("Expected a number but got {}", v))),
        }
    }

    fn internal_error(&self, msg: &str) -> ! {
        panic!("Internal error: {}", msg)
    }

    fn binary(&mut self, operator: &OpCode) -> Result<()> {
        let b = self.pop_number()?;
        let a = self.pop_number()?;
        self.stack.push(Value::Number(match operator {
            OpCode::Subtract => a + b,
            OpCode::Multiply => a * b,
            OpCode::Divide => a / b,
            _ => unreachable!(),
        }));
        Ok(())
    }

    fn runtime_error(&self, msg: &str) -> Error {
        Error::from(format!(
            "Runtime error : {} at {}",
            msg,
            self.chunk.dissassemble_ins(self.ip)
        ))
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
        chunk.add_const_ins(Value::Number(1.2), 123);
        chunk.add_const_ins(Value::Number(3.4), 123);
        chunk.write_ins(OpCode::Add, 123);
        chunk.write_ins(OpCode::Return, 123);

        let mut vm = VM::with_chunk(&chunk);
        assert!(matches!(vm.run(), Ok(())));
    }
    #[test]
    fn test_binary_ops() {
        let mut chunk = Chunk::new();

        chunk.add_const_ins(Value::Number(5.), 123);
        chunk.add_const_ins(Value::Number(3.), 123);
        chunk.write_ins(OpCode::Add, 123);
        chunk.add_const_ins(Value::Number(4.), 123);
        chunk.write_ins(OpCode::Divide, 123);
        chunk.write_ins(OpCode::Negate, 123);
        chunk.add_const_ins(Value::Number(2.5), 123);

        chunk.write_ins(OpCode::Multiply, 123);

        let mut vm = VM::with_chunk(&chunk);
        assert!(matches!(vm.run(), Ok(())));
        assert_eq!(vm.stack, vec![Value::Number(-5.)]);
    }
}
