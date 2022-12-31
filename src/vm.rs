use crate::chunk::{Chunk, OpCode};

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
                self.chunk.disassemble_code(self.ip);
            }
            match op_code {
                OpCode::Constant { value, idx: _ } => println!("{value}"),
                OpCode::ConstantLong { value, idx: _ } => println!("{value}"),
                OpCode::Return => break,
            }
            self.ip += op_code.code_size()
        }
        Ok(())
    }
}

pub enum InterpretError {
    CompileError,
    RuntimeError,
}
