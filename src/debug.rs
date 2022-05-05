use crate::chunk::Chunk;
use crate::chunk::OpCode;

pub fn disassemble_chunk(_chunk: &Chunk, name: &str) {
    println!("== {} ==", name);

    for (offset, op_code) in _chunk.code.iter().enumerate() {
        let line;
        if offset > 0 && _chunk.lines[offset] == _chunk.lines[offset - 1] {
            line = String::from("|");
        } else {
            line = format!("{}", _chunk.lines[offset]);
        }

        println!(
            "{offset:0>4} {line: >4} {instruction}",
            offset = offset,
            line = line,
            instruction = disassemble_instruction(&_chunk, op_code)
        );
    }
}

fn disassemble_instruction(_chunk: &Chunk, op_code: &OpCode) -> String {
    match op_code {
        OpCode::Return => String::from("OP_RETURN"),
        OpCode::Constant(idx) => {
            format!("OP_CONSTANT         {} '{}'", idx, _chunk.constants[*idx])
        }
    }
}
