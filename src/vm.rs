use std::io;
use std::io::Write;
use std::{collections::HashMap, rc::Rc};

use crate::bytecode::FunctionObj;
use crate::{
    bytecode::{self, OpCode, Value},
    Error, Result,
};

struct CallFrame {
    ip: usize,
    stack_start: usize,
    fun_id: usize,
}

impl CallFrame {
    fn new(stack_start: usize, fun_id: usize) -> Self {
        Self {
            ip: 0,
            stack_start,
            fun_id,
        }
    }
}

pub struct VM<'a> {
    frames: Vec<CallFrame>,
    lock: io::StdoutLock<'a>,
    stack: Vec<bytecode::Value>,
    functions: Vec<FunctionObj>,
    globals: HashMap<Rc<String>, Value>,
}

impl<'a> VM<'a> {
    const FRAME_MAX: usize = 256;
    const STACK_MAX: usize = 256;
    pub fn new(functions: Vec<FunctionObj>) -> Self {
        let mut stack = Vec::with_capacity(Self::STACK_MAX);

        let code_id = functions.len() - 1;
        let code = Value::Function(code_id);
        stack.push(code);

        let frame = CallFrame::new(0, code_id);
        Self {
            frames: vec![frame],
            lock: io::stdout().lock(),
            functions,
            stack,
            globals: HashMap::new(),
        }
    }

    fn frame_stack(&self) -> &[Value] {
        &self.stack[self.curr_frame().stack_start..]
    }

    fn frame_stack_mut(&mut self) -> &mut [Value] {
        let start = self.curr_frame().stack_start;
        &mut self.stack[start..]
    }

    fn stack_get(&self, index: usize) -> &Value {
        &self.stack[self.curr_frame().stack_start + index]
    }
    fn stack_get_mut(&mut self, index: usize) -> &mut Value {
        let start = self.curr_frame().stack_start;
        &mut self.stack[start + index]
    }

    fn curr_frame(&self) -> &CallFrame {
        // theres always a frame, because all the code is wrapped in implicit 1main function
        self.frames.last().unwrap()
    }

    fn curr_frame_mut(&mut self) -> &mut CallFrame {
        self.frames.last_mut().unwrap()
    }

    fn chunk(&self) -> &bytecode::Chunk {
        let id = self.frames.last().unwrap().fun_id;
        self.get_fun(id).chunk()
    }

    fn get_fun(&self, id: usize) -> &FunctionObj {
        &self.functions[id]
    }

    fn ip(&self) -> usize {
        self.curr_frame().ip
    }

    fn ip_mut(&mut self) -> &mut usize {
        &mut self.curr_frame_mut().ip
    }

    fn _trace(&mut self) {
        // writeln!(self.lock, "stack: {:?}", self.stack).unwrap();
        write!(self.lock, "stack: [").unwrap();
        for (i, v) in self.stack.iter().enumerate() {
            if i == self.curr_frame().stack_start {
                write!(self.lock, " | ").unwrap();
            }
            write!(self.lock, "{}, ", v).unwrap();
        }
        writeln!(self.lock, "]").unwrap();
        writeln!(
            self.lock,
            "ins:   {}\n",
            self.chunk().dissassemble_ins(self.ip())
        )
        .unwrap();
    }

    pub fn run(&mut self) -> crate::Result<()> {
        #[cfg(feature = "trace")]
        writeln!(self.lock, "=== TRACE ===").unwrap();

        #[cfg(feature = "bench")]
        let start = std::time::Instant::now();

        while !self.is_at_end() {
            // #[cfg(feature = "trace")]
            // self._trace();

            let _return = self.execute_ins()?;
            if _return {
                break;
            }
        }

        if !self.stack.len() == 1 {
            eprintln!("WARNING: stack is not empty at the end of execution");
        }

        #[cfg(feature = "bench")]
        writeln!(
            self.lock,
            "=== BENCH ===\nelapsed time:{:?}",
            start.elapsed()
        );

        Ok(())
    }

    fn execute_ins(&mut self) -> Result<bool> {
        // return if we want to stop execution
        match *self.advance() {
            OpCode::Constant(index) => self.add_const(index),
            OpCode::Print => self.print()?,
            OpCode::Pop => {
                self.pop_stack();
            }
            OpCode::DefineGlobal(index) => self.define_global(index),
            OpCode::GetGlobal(index) => self.get_global(index)?,
            OpCode::SetGlobal(index) => self.set_global(index)?,

            OpCode::GetLocal(offset) => self.push_stack(self.stack_get(offset as usize).clone()),
            OpCode::SetLocal(offset) => {
                *self.stack_get_mut(offset as usize) = self.peek_stack_unwrapped(0).clone();
            }

            OpCode::JumpIfFalse(offset) => {
                let offset =
                    offset.expect("Internal error: jump instruction has no offset") as usize;
                if !self.peek_stack_unwrapped(0).is_truthy() {
                    *self.ip_mut() += offset;
                }
            }
            OpCode::Jump(offset) => {
                let offset =
                    offset.expect("Internal error: jump instruction has no offset") as usize;
                *self.ip_mut() += offset;
            }

            OpCode::Loop(offset) => {
                *self.ip_mut() -= offset as usize;
            }

            OpCode::Call(arg_count) => self.call(arg_count)?,

            OpCode::True => self.push_stack(Value::Boolean(true)),
            OpCode::False => self.push_stack(Value::Boolean(false)),
            OpCode::Nil => self.push_stack(Value::Nil),

            OpCode::Negate => self.negate()?,
            OpCode::Not => self.not()?,

            op @ (OpCode::Greater | OpCode::Less) => self.comparison(op)?,
            OpCode::Equal => self.equality(),

            OpCode::Add => self.add()?,
            op @ (OpCode::Subtract | OpCode::Multiply | OpCode::Divide) => self.binary(op)?,

            OpCode::Return => {
                let ret = self.pop_stack();
                let frame = self.frames.pop().unwrap();

                let stack_start = frame.stack_start;

                if self.frames.is_empty() {
                    self.pop_stack();
                    return Ok(true);
                }
                self.stack.truncate(stack_start);
                self.push_stack(ret);
            }
        }
        Ok(false)
    }

    fn call(&mut self, arg_count: u8) -> Result<()> {
        let calee = self.peek_stack_unwrapped(arg_count as usize);
        match calee {
            Value::Function(id) => {
                let f = self.get_fun(*id);
                if arg_count != f.arity() {
                    return Err(self.runtime_error(&format!(
                        "Expected {} arguments but got {} in call to {}()",
                        f.arity(),
                        arg_count,
                        f.name()
                    )));
                }
                if self.frames.len() == Self::FRAME_MAX {
                    return Err(self.runtime_error("Stack overflow"));
                }
                let frame = CallFrame::new(self.stack.len() - arg_count as usize - 1, *id);
                self.frames.push(frame);
            }
            _ => return Err(self.runtime_error(&format!("Can only call functions, not {}", calee))),
        }

        Ok(())
    }

    fn define_global(&mut self, index: u16) {
        if let Value::String(s) = self.chunk().get_const(index) {
            let ident = Rc::clone(s);
            let val = self.pop_stack();
            self.globals.insert(ident, val);
        } else {
            panic!("define global: expected string")
        }
    }

    fn get_global(&mut self, index: u16) -> Result<()> {
        if let Value::String(ident) = self.chunk().get_const(index) {
            let val = self.globals.get(ident).ok_or_else(|| {
                self.runtime_error(&format!("Undefined global variable '{ident}'"))
            })?;
            let val = val.clone();
            self.push_stack(val);
            Ok(())
        } else {
            self.internal_error("get global: expected string")
        }
    }

    fn set_global(&mut self, index: u16) -> Result<()> {
        if let Value::String(s) = self.chunk().get_const(index) {
            let ident = Rc::clone(s);
            let val = self.peek_stack_unwrapped(0).clone();
            let present = self.globals.contains_key(&ident);
            if present {
                self.globals.insert(ident, val);
                Ok(())
            } else {
                Err(self.runtime_error(&format!("Undefined global variable '{ident}'")))
            }
        } else {
            self.internal_error("set global: expected string")
        }
    }

    fn print(&mut self) -> Result<()> {
        let val = self.pop_stack();
        writeln!(self.lock, "{}", val)?;
        Ok(())
    }

    fn add_const(&mut self, id: u16) {
        // todo- remove that clone
        self.push_stack(self.chunk().get_const(id).clone());
    }

    fn push_stack(&mut self, v: Value) {
        self.stack.push(v)
    }

    fn negate(&mut self) -> Result<()> {
        match self.pop_stack() {
            Value::Number(n) => self.push_stack(Value::Number(-n)),
            v => return Err(self.runtime_error(&format!("Cannot negate {v}"))),
        };
        Ok(())
    }

    fn not(&mut self) -> Result<()> {
        match self.pop_stack() {
            Value::Boolean(b) => self.push_stack(Value::Boolean(!b)),
            Value::Nil => self.push_stack(Value::Boolean(true)),
            v => return Err(self.runtime_error(&format!("Cannot perform '!' operation on {v}"))),
        }
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
                self.stack.push(Value::String(Rc::new(format!("{s1}{s2}"))));
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
        if offset < self.stack.len() {
            self.stack.get(self.stack.len() - 1 - offset)
        } else {
            None
        }
    }

    fn peek_stack_unwrapped(&self, offset: usize) -> &Value {
        if let Some(v) = self.peek_stack(offset) {
            v
        } else {
            self.internal_error(&format!(
                "Expected value at stack at index {} but found no value",
                self.stack.len() as isize - 1 - offset as isize
            ))
        }
    }

    // fn peek_stack_mut(&mut self, offset: usize) -> Option<&mut Value> {
    //     self.stack.get_mut(self.stack.len() - 1 - offset)
    // }

    fn peek_number(&self, offset: usize) -> Result<&f64> {
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

    fn binary(&mut self, operator: OpCode) -> Result<()> {
        let b = self.pop_number()?;
        let a = self.pop_number()?;
        self.stack.push(Value::Number(match operator {
            OpCode::Subtract => a - b,
            OpCode::Multiply => a * b,
            OpCode::Divide => a / b,
            _ => unreachable!(),
        }));
        Ok(())
    }

    fn comparison(&mut self, operator: OpCode) -> Result<()> {
        let b = self.pop_number()?;
        let a = self.pop_number()?;
        self.push_stack(Value::Boolean(match operator {
            OpCode::Greater => a > b,
            OpCode::Less => a < b,
            _ => unreachable!(),
        }));
        Ok(())
    }

    fn equality(&mut self) {
        let b = self.pop_stack();
        let a = self.pop_stack();

        self.push_stack(Value::Boolean(a == b))
    }

    fn runtime_error(&self, msg: &str) -> Error {
        let mut full_msg = format!("Runtime error: {} \nstack trace:", msg);

        for frame in self.frames.iter().rev() {
            let func = self.get_fun(frame.fun_id);
            let line = func.chunk().get_line(frame.ip - 1);
            full_msg.push_str(&format!("\n[line {}] in {}()", line, func.name()));
        }

        return Error::from(full_msg);
    }

    fn is_at_end(&self) -> bool {
        self.ip() >= self.chunk().code().len()
    }

    fn advance(&mut self) -> &bytecode::OpCode {
        *self.ip_mut() += 1;
        &self.chunk().code()[self.ip() - 1]
    }
}
