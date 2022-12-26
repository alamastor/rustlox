use crate::chunk::Chunk;
use crate::chunk::OpCode;
use crate::debug;

pub struct VM<'a> {
    chunk: &'a Chunk,
    ip: *const u8,
}

pub fn interpret(chunk: Chunk) -> Result<(), String> {
    let vm = VM {
        chunk: &chunk,
        ip: chunk.code.as_ptr(),
    };
    let mut idx = 0;
    loop {
        if cfg!(debug_trace_execution) {
            let instruction = debug::disassemble_instruction(&chunk, vm.ip.add(idx) as &u8, offset);
            println!(
                "{offset:0>4} {line: >4} {instruction}",
                offset = offset,
                line = line,
                instruction = instruction
            );
        }
        match unsafe { OpCode::from(*vm.ip.add(idx)) } {
            OpCode::Constant => println!("{}", chunk.read_constant(unsafe { *vm.ip as usize })),
            OpCode::Return => break Ok(()),
        }
        idx += 1;
    }
}
