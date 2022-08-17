use rlox::bytecode::{Chunk, OpCode};

fn main() {
    let mut chunk = Chunk::new();
    
    chunk.write_chunk(OpCode::Constant(0), 123);
    chunk.add_const(1.0);
    chunk.write_chunk(OpCode::Return, 123);

    println!("{}", chunk);
}
