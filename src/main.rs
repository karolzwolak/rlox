use rlox::{bytecode::{Chunk, OpCode}, vm::VM, vm};

fn main() {
    let mut chunk = Chunk::new();
    
    chunk.write_chunk(OpCode::Constant(0), 123);
    chunk.add_const(1.0);
    chunk.write_chunk(OpCode::Return, 123);

    println!("chunk :\n{}", chunk);

    let mut vm = VM::with_chunk(&chunk, vm::ExecutionMode::Debug);
    vm.interpret();

}
