use crate::chunk::{Chunk, OpCode};
use crate::debug::format_op_code;

pub fn interpret(chunk: &Chunk) -> Result<(), InterpretError> {
    let mut vm = VM { chunk, ip: 0 };
    vm.run()
}

pub struct VM<'a> {
    chunk: &'a Chunk,
    ip: usize,
}
impl<'a> VM<'a> {
    fn run(&mut self) -> Result<(), InterpretError> {
        loop {
            let op_code = self.chunk.decode(self.ip);
            if cfg!(feature = "trace") {
                println!("{}", format_op_code(&op_code));
            }
            match op_code {
                OpCode::Constant { value, idx: _ } => println!("{value}"),
                OpCode::ConstantLong { value, idx: _ } => println!("{value}"),
                OpCode::Return => break,
            }
            self.ip += 1
        }
        Ok(())
    }
}

pub enum InterpretError {
    CompileError,
    RuntimeError,
}
