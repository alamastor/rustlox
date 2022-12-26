mod chunk;
mod debug;

use chunk::OpCode;
use debug::Disassemble;

fn main() {
    let chunk = vec![OpCode::Return];
    chunk.disassemble("test chunk")
}
