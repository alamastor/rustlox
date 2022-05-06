use crate::chunk::Chunk;
use crate::chunk::OpCode;

pub fn disassemble_chunk(_chunk: &Chunk, name: &str) {
    println!("== {} ==", name);

    let mut offset = 0;
    while offset < _chunk.code.len() {
        let op_code = &_chunk.code[offset];
        let line;
        if offset > 0 && _chunk.get_line(offset) == _chunk.get_line(offset - 1) {
            line = String::from("|");
        } else {
            line = format!("{}", _chunk.get_line(offset));
        }
        let instruction = disassemble_instruction(&_chunk, op_code, offset);
        println!(
            "{offset:0>4} {line: >4} {instruction}",
            offset = offset,
            line = line,
            instruction = instruction
        );
        offset += OpCode::from(*op_code).size();
    }
}

fn disassemble_instruction(_chunk: &Chunk, op_code: &u8, offset: usize) -> String {
    match OpCode::from(*op_code) {
        OpCode::Return => String::from("OP_RETURN"),
        OpCode::Constant => format!(
            "OP_CONSTANT         {} '{}'",
            offset,
            _chunk.constants[_chunk.code[offset + 1] as usize]
        ),
    }
}
