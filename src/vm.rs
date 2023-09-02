use crate::chunk::{Chunk, Op};
use crate::compiler;
use crate::object::{Object, Objects};
use crate::value::Value;
use std::io::Write;
use std::slice::Iter;

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
            let a = *a;
            let b = *b;
            $self.pop();
            $self.pop();

            $self.push(Value::Number(b $op a));
        } else {
            $self.runtime_error("Operands must be numbers.");
        }
    };
}

macro_rules! bool_bin_op {
    ($self:ident, $op:tt) => {
        if let Value::Number(a) = $self.peek(0) && let Value::Number(b) = $self.peek(1) {
            let a = *a;
            let b = *b;
            $self.pop();
            $self.pop();

            $self.push(Value::Bool(a $op b));
        } else {
            $self.runtime_error("Operands must be numbers.");
        }
    };
}

pub struct VM<'a, O: Write, E: Write> {
    chunk: &'a Chunk,
    ip: usize,
    stack: Vec<Value>,
    objects: Objects,
    out_stream: &'a mut O,
    err_stream: &'a mut E,
}
impl<'a, O: Write, E: Write> VM<'a, O, E> {
    fn new(chunk: &'a Chunk, out_stream: &'a mut O, err_stream: &'a mut E) -> VM<'a, O, E> {
        VM {
            chunk,
            ip: 0,
            stack: vec![],
            objects: Objects::new(),
            out_stream,
            err_stream,
        }
    }

    fn run(&mut self) -> Result<(), InterpretError> {
        loop {
            let (op, op_size) = self.chunk.decode(self.ip);
            if cfg!(feature = "trace") {
                self.chunk.disassemble_code(self.ip);
                print!("          ");
                for val in self.iter() {
                    print!("[{val}]");
                }
                println!();
            }
            match op {
                Op::Constant { value } => self.push(value),
                Op::Return => {
                    let return_val = self.pop();
                    writeln!(self.out_stream, "{return_val}").unwrap();
                    return Result::Ok(());
                }
                Op::Negate => {
                    match self.peek(0) {
                        Value::Number(val) => {
                            let val = *val;
                            self.pop();
                            self.stack.push(Value::Number(-val));
                        }
                        _ => {
                            self.runtime_error("Operand must be a number.");
                            return Result::Err(InterpretError::RuntimeError);
                        }
                    };
                }
                Op::Nil => {
                    self.push(Value::Nil);
                }
                Op::True => {
                    self.push(Value::Bool(true));
                }
                Op::False => {
                    self.push(Value::Bool(false));
                }
                Op::Add => {
                    if let Value::Obj(x) = self.peek(0) &&
                       let Object::String { chars: a } = &**x &&
                       let Value::Obj(y) = self.peek(1)
                    {
                        let Object::String { chars: b } = &**y;
                        let new_string = self.objects.new_string(b.to_owned() + a);
                        self.stack.push(Value::Obj(new_string));
                    } else {
                        bin_op!(self, +);
                    }
                }
                Op::Subtract => {
                    bin_op!(self, -);
                }
                Op::Multiply => {
                    bin_op!(self, *);
                }
                Op::Divide => {
                    bin_op!(self, /);
                }
                Op::Not => {
                    let bool = Value::Bool(is_falsey(self.pop()));
                    self.stack.push(bool);
                }
                Op::Equal => {
                    let a = self.pop();
                    let b = self.pop();
                    self.push(Value::Bool(values_equal(a, b)))
                }
                Op::Greater => {
                    bool_bin_op!(self, >);
                }
                Op::Less => {
                    bool_bin_op!(self, <);
                }
            }
            self.ip += op_size;
        }
    }

    fn push(&mut self, value: Value) {
        if cfg!(feature = "trace") {
            println!("Pushing {value}");
        }
        self.stack.push(value);
    }

    fn pop(&mut self) -> Value {
        match self.stack.pop() {
            Some(x) => x,
            None => panic!("Tried to pop an empty stack!"),
        }
    }

    fn peek(&self, distance: usize) -> &Value {
        &self.stack[self.stack.len() - 1 - distance]
    }

    fn iter(&self) -> Iter<'_, Value> {
        self.stack.iter()
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
        Value::Obj(_) => a == b,
    }
}
