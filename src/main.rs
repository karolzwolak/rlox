use rlox::{
    bytecode::{Chunk, OpCode},
    vm::VM,
};

fn main() {
    let mut chunk = Chunk::new();
    
    chunk.add_const_ins(5., 123);
    chunk.add_const_ins(3., 123);
    chunk.write_ins(OpCode::Add, 123);
    chunk.add_const_ins(4., 123);
    chunk.write_ins(OpCode::Divide, 123);
    chunk.write_ins(OpCode::Negate, 123);
    chunk.add_const_ins(2.5, 123);
    chunk.write_ins(OpCode::Multiply, 123);
    chunk.write_ins(OpCode::Return, 123);

    let mut vm = VM::with_chunk(&chunk);
    vm.interpret();
}
