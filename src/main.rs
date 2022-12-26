mod chunk;
mod debug;

use chunk::Chunk;

fn main() {
    let mut chunk = Chunk::new();
    chunk.add_const(1.2);
    chunk.add_return();
    chunk.disassemble("test chunk")
}
