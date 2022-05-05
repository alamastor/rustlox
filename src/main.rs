mod chunk;
mod debug;

use chunk::Chunk;
use chunk::OpCode;

fn main() {
    let mut _chunk = Chunk::new();

    let constant = _chunk.add_constant(1.2);
    _chunk.write(OpCode::Constant(constant), 123);
    _chunk.write(OpCode::Return, 123);

    debug::disassemble_chunk(&_chunk, "test chunk");
}
