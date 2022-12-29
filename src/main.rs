mod chunk;
mod debug;

use chunk::{Chunk, Op};

fn main() {
    let mut chunk = Chunk::new();
    chunk.push_op_code(Op::Constant {
        value: 1.2,
        extras: None,
    });
    chunk.push_op_code(Op::Return {});
    chunk.disassemble("test chunk")
}
