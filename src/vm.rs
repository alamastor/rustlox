use crate::chunk::{Chunk, OpCode};
use crate::compiler;
use crate::value::Value;
use std::io::Write;

pub fn interpret<O: Write, E: Write>(
    source: &str,
    out_stream: &mut O,
    err_stream: &mut E,
) -> Result<(), InterpretError> {
    let chunk = compiler::compile(source).map_err(|_| InterpretError::CompileError)?;
    VM::new(&chunk, out_stream, err_stream).run()
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

macro_rules! bool_bin_op {
    ($self:ident, $op:tt) => {
        if let Value::Number(a) = $self.peek(0) && let Value::Number(b) = $self.peek(1) {
            $self.pop();
            $self.pop();

            $self.stack.push(Value::Bool(a $op b));
        } else {
            $self.runtime_error("Operands must be numbers.");
        }
    };
}

pub struct VM<'a, O: Write, E: Write> {
    chunk: &'a Chunk,
    ip: usize,
    stack: Vec<Value>,
    out_stream: &'a mut O,
    err_stream: &'a mut E,
}
impl<'a, O: Write, E: Write> VM<'a, O, E> {
    fn new(chunk: &'a Chunk, out_stream: &'a mut O, err_stream: &'a mut E) -> VM<'a, O, E> {
        VM {
            chunk,
            ip: 0,
            stack: vec![],
            out_stream,
            err_stream,
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
                    let return_val = self.pop();
                    writeln!(self.out_stream, "{return_val}").unwrap();
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
                OpCode::Nil => {
                    self.stack.push(Value::Nil);
                }
                OpCode::True => {
                    self.stack.push(Value::Bool(true));
                }
                OpCode::False => {
                    self.stack.push(Value::Bool(false));
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
                OpCode::Not => {
                    let bool = Value::Bool(is_falsey(self.pop()));
                    self.stack.push(bool);
                }
                OpCode::Equal => {
                    let a = self.pop();
                    let b = self.pop();
                    self.stack.push(Value::Bool(values_equal(a, b)))
                }
                OpCode::Greater => {
                    bool_bin_op!(self, >);
                }
                OpCode::Less => {
                    bool_bin_op!(self, <);
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

#[derive(Debug)]
pub enum InterpretError {
    CompileError,
    RuntimeError,
}

fn is_falsey(value: Value) -> bool {
    match value {
        Value::Nil => true,
        Value::Bool(x) => !x,
        _ => false,
    }
}

fn values_equal(a: Value, b: Value) -> bool {
    match a {
        Value::Bool(_) => a == b,
        Value::Nil => true,
        Value::Number(_) => a == b,
    }
}
