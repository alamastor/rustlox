use crate::chunk::{Chunk, OpCode};

pub fn interpret(chunk: &Chunk) {
    let mut vm = VM {
        chunk,
        ip: 0,
        stack: vec![],
    };
    vm.run()
}

macro_rules! bin_op {
    ($self:ident, $op:tt) => {
        let b = $self.pop();
        let a = $self.pop();
        $self.stack.push(a $op b);
    };
}

pub struct VM<'a> {
    chunk: &'a Chunk,
    ip: usize,
    stack: Vec<f64>,
}
impl<'a> VM<'a> {
    fn run(&mut self) {
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
                    println!("{}", self.pop());
                    break;
                }
                OpCode::Negate => {
                    let val = self.pop();
                    self.stack.push(-val)
                }
                OpCode::Add => {
                    bin_op!(self, +);
                }
                OpCode::Subtract => {
                    bin_op!(self, -);
                }
                OpCode::Multiply => {
                    bin_op!(self, *);
                }
                OpCode::Divide => {
                    bin_op!(self, /);
                }
            }
            self.ip += op_code.code_size()
        }
    }

    fn pop(&mut self) -> f64 {
        match self.stack.pop() {
            Some(x) => x,
            None => panic!("Tried to pop an empty stack!"),
        }
    }
}
