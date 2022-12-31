mod chunk;
mod debug;

use chunk::{Chunk, Op};

fn main() {
    let mut chunk = Chunk::new();
    chunk.push_op_code(
        Op::Constant {
            value: 1.2,
            extras: None,
        },
        123,
    );
    chunk.push_op_code(Op::Return {}, 123);
    chunk.disassemble("test chunk")
}
