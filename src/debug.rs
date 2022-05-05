use crate::chunk::Chunk;
use crate::chunk::OpCode;
use std::convert::TryInto;

pub fn disassemble_chunk(_chunk: &Chunk, name: &str) {
    println!("== {} ==", name);

    let mut offset = 0;
    while offset < _chunk.code.len() {
        let op_code = &_chunk.code[offset];
        let line;
        if offset > 0 && _chunk.lines[offset] == _chunk.lines[offset - 1] {
            line = String::from("|");
        } else {
            line = format!("{}", _chunk.lines[offset]);
        }
        let instruction = disassemble_instruction(&_chunk, op_code, offset);
        println!(
            "{offset:0>4} {line: >4} {instruction}",
            offset = offset,
            line = line,
            instruction = instruction
        );
        offset += instruction_len(op_code);
    }
}

fn disassemble_instruction(_chunk: &Chunk, op_code: &u8, offset: usize) -> String {
    match (*op_code).try_into() {
        Ok(OpCode::Return) => String::from("OP_RETURN"),
        Ok(OpCode::Constant) => format!(
            "OP_CONSTANT         {} '{}'",
            offset + 1,
            _chunk.constants[_chunk.code[offset + 1] as usize]
        ),
        Err(msg) => panic!(msg),
    }
}

fn instruction_len(op_code: &u8) -> usize {
    match (*op_code).try_into() {
        Ok(OpCode::Return) => 1,
        Ok(OpCode::Constant) => 2,
        Err(msg) => panic!(msg),
    }
}
