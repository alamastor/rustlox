use crate::chunk::{Chunk, OpCode};

pub fn interpret(chunk: &Chunk) -> Result<(), InterpretError> {
    let mut vm = VM {
        chunk,
        ip: 0,
        stack: vec![],
    };
    vm.run()
}

pub struct VM<'a> {
    chunk: &'a Chunk,
    ip: usize,
    stack: Vec<f64>,
}
impl<'a> VM<'a> {
    fn run(&mut self) -> Result<(), InterpretError> {
        loop {
            let op_code = self.chunk.decode(self.ip);
            if cfg!(feature = "trace") {
                self.chunk.disassemble_code(self.ip);
                print!("          ");
                for val in self.stack.iter() {
                    print!("[{val}]");
                }
                print!("\n");
            }
            match op_code {
                OpCode::Constant { value, idx: _ } => self.stack.push(value),
                OpCode::ConstantLong { value, idx: _ } => self.stack.push(value),
                OpCode::Return => {
                    match self.stack.pop() {
                        Some(x) => println!("{x}"),
                        None => panic!("Tried to pop an empty stack!"),
                    }
                    break;
                }
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
