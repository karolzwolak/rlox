use rlox::{bytecode::{Chunk, OpCode}, vm::VM};

fn main() {
    let mut chunk = Chunk::new();
    
    chunk.write_chunk(OpCode::Constant(0), 123);
    chunk.add_const(1.0);
    chunk.write_chunk(OpCode::Constant(1), 123);
    chunk.add_const(2.0);
    chunk.write_chunk(OpCode::Return, 123);

    let mut vm = VM::with_chunk(&chunk);
    vm.interpret();

}
