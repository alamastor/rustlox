#![feature(trace_macros)]
mod chunk;
mod debug;
mod vm;

use chunk::{Chunk, Op};

fn main() {
    let mut chunk = Chunk::new();
    chunk.push_op_code(Op::Constant { value: 1.2 }, 123);
    chunk.push_op_code(Op::Constant { value: 3.4 }, 123);
    chunk.push_op_code(Op::Add, 123);
    chunk.push_op_code(Op::Constant { value: 5.6 }, 123);
    chunk.push_op_code(Op::Divide, 123);
    chunk.push_op_code(Op::Negate, 123);
    chunk.push_op_code(Op::Constant { value: -0.8123 }, 123);
    chunk.push_op_code(Op::Subtract, 123);
    chunk.push_op_code(Op::Constant { value: 10_000.0 }, 123);
    chunk.push_op_code(Op::Multiply, 123);
    chunk.push_op_code(Op::Return {}, 123);
    chunk.disassemble("test chunk");
    vm::interpret(&chunk)
}
