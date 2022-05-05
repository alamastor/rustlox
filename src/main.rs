mod chunk;
mod debug;

use chunk::Chunk;
use chunk::OpCode;

fn main() {
    let mut _chunk = Chunk::new();

    let constant = _chunk.add_constant(1.2);
    _chunk.write(OpCode::Constant as u8, 123);
    _chunk.write(constant, 123);
    _chunk.write(OpCode::Return as u8, 123);

    debug::disassemble_chunk(&_chunk, "test chunk");
}
