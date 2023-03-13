use crate::chunk::{Chunk, OpCode};
use crate::compiler;
use crate::value::Value;

pub fn interpret(source: &str) -> Result<(), InterpretError> {
    let chunk = compiler::compile(source).map_err(|_| InterpretError::CompileError)?;
    VM::new(&chunk).run()
}

macro_rules! bin_op {
    ($self:ident, $op:tt) => {
        if let Value::Number(a) = $self.peek(0) && let Value::Number(b) = $self.peek(1) {
            $self.pop();
            $self.pop();
            $self.stack.push(Value::Number(a $op b));
        } else {
            $self.runtime_error("Operands must be numbers.");
        }
    };
}

pub struct VM<'a> {
    chunk: &'a Chunk,
    ip: usize,
    stack: Vec<Value>,
}
impl<'a> VM<'a> {
    fn new(chunk: &Chunk) -> VM {
        VM {
            chunk,
            ip: 0,
            stack: vec![],
        }
    }
    fn run(&mut self) -> Result<(), InterpretError> {
        loop {
            let op_code = self.chunk.decode(self.ip);
            if cfg!(feature = "trace") {
                self.chunk.disassemble_code(self.ip);
                print!("          ");
                for val in self.stack.iter() {
                    print!("[{val}]");
                }
                println!();
            }
            match op_code {
                OpCode::Constant { value, idx: _ } => self.stack.push(value),
                OpCode::ConstantLong { value, idx: _ } => self.stack.push(value),
                OpCode::Return => {
                    println!("{}", self.pop());
                    return Result::Ok(());
                }
                OpCode::Negate => {
                    match self.peek(0) {
                        Value::Number(val) => {
                            self.pop();
                            self.stack.push(Value::Number(-val));
                        }
                        _ => {
                            self.runtime_error("Operand must be a number.");
                            return Result::Err(InterpretError::RuntimeError);
                        }
                    };
                }
                OpCode::Nil => {self.stack.push(Value::Nil);}
                OpCode::True => {self.stack.push(Value::Bool(true));}
                OpCode::False => {self.stack.push(Value::Bool(false));}
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
                OpCode::Not => {
                    let bool = Value::Bool(is_falsey(self.pop()));
                    self.stack.push(bool);
                }
            }
            self.ip += op_code.code_size()
        }
    }

    fn pop(&mut self) -> Value {
        match self.stack.pop() {
            Some(x) => x,
            None => panic!("Tried to pop an empty stack!"),
        }
    }

    fn peek(&self, distance: usize) -> Value {
        self.stack[self.stack.len() - 1 - distance]
    }

    fn runtime_error(&mut self, message: &str) {
        eprintln!("{message}");
        let line = self.chunk.get_line_no(self.chunk.get_op_idx(self.ip - 1));
        eprintln!("[line {line}] in script");
        self.stack = vec![];
    }

}

pub enum InterpretError {
    CompileError,
    RuntimeError,
}

fn is_falsey(value: Value) -> bool {
    match value {
        Value::Nil => true,
        Value::Bool(x) => !x,
        _ => false
    }
}
