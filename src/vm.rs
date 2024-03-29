use crate::chunk::{Chunk, Op};
use crate::compiler;
use crate::object::{Function, Object};
use crate::strings::Strings;
use crate::value::Value;
use std::cmp;
use std::collections::HashMap;
use std::fmt::Write as FmtWrite;
use std::io::Write;
use std::rc::Rc;
use std::slice::Iter;

pub fn interpret<O: std::io::Write, E: std::io::Write>(
    source: &str,
    out_stream: &mut O,
    err_stream: &mut E,
) -> Result<(), InterpretError> {
    let (function, objects, strings) =
        compiler::compile(source).map_err(|_| InterpretError::CompileError)?;
    VM::new(function, objects, strings, out_stream, err_stream).run()
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
            $self.runtime_error("Operands must be numbers.".to_string());
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

            $self.push(Value::Bool(b $op a));
        } else {
            $self.runtime_error("Operands must be numbers.".to_string());
        }
    };
}

pub struct VM<'a, O: Write, E: Write> {
    frames: Vec<CallFrame>,
    objects: Vec<Object>,
    strings: Strings,
    globals: HashMap<Rc<String>, Value>,
    out_stream: &'a mut O,
    err_stream: &'a mut E,
}
impl<'a, O: Write, E: Write> VM<'a, O, E> {
    fn new(
        function: Function,
        objects: Vec<Object>,
        strings: Strings,
        out_stream: &'a mut O,
        err_stream: &'a mut E,
    ) -> VM<'a, O, E> {
        VM {
            objects,
            strings,
            frames: vec![CallFrame::new(function, vec![])],
            globals: HashMap::new(),
            out_stream,
            err_stream,
        }
    }

    fn run(&mut self) -> Result<(), InterpretError> {
        loop {
            let (op, op_size) = self
                .current_frame()
                .function
                .chunk
                .decode(self.current_frame().ip);
            if cfg!(feature = "trace") {
                self.current_frame()
                    .function
                    .chunk
                    .disassemble_code(self.current_frame().ip);
                print!("          ");
                for val in self.iter() {
                    print!("[{val}]");
                }
                println!();
            }
            let mut ip_offset = 0;
            match op {
                Op::Constant { value } => self.push(value),
                Op::Return => {
                 return Result::Ok(());
                }
                Op::Print => {
                    let val = self.pop();
                    writeln!(self.out_stream, "{val}").unwrap();
                }
                Op::Negate => {
                    match self.peek(0) {
                        Value::Number(val) => {
                            let val = *val;
                            self.pop();
                            self.current_frame_mut().slots.push(Value::Number(-val));
                        }
                        _ => {
                            return Result::Err(self.runtime_error("Operand must be a number.".to_string()));
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
                Op::Pop => {
                    self.pop();
                }
                Op::GetLocal { idx } => {
                    self.push(self.current_frame().slots[idx as usize].clone());
                }
                Op::SetLocal { idx } => {
                    self.current_frame_mut().slots[idx as usize] = self.peek(0).clone();
                }
                Op::GetGlobal { name } => {
                    match self.globals.get(&name) {
                        Some(value) => self.push(value.to_owned()),
                        None => {
                            return Result::Err(self.runtime_error(format!("Undefined variable '{}'.", name)));
                        }
                    }
                }
                Op::DefineGlobal { name } => {
                    let val = self.pop();
                    self.globals.insert(name, val);
                }
                Op::SetGlobal { name } => {
                    if self.globals.contains_key(&*name) {
                        self.globals.insert(name, self.peek(0).clone());
                    } else {
                        return Result::Err(self.runtime_error(format!("Undefined variable '{}'.", name)));
                    }
                }
                Op::Add => {
                    if let Value::Obj(x) = self.peek(0) &&
                       let Object::String { chars: a } = x &&
                       let Value::Obj(y) = self.peek(1) &&
                        let Object::String { chars: b } = y
                    {
                        let new_string = self.strings.new_string((**b).to_owned() + &**a);
                        self.current_frame_mut().slots.push(Value::Obj(Object::String { chars: new_string }));
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
                    let bool = Value::Bool(is_falsey(&self.pop()));
                    self.current_frame_mut().slots.push(bool);
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
                Op::JumpIfFalse { offset } => {
                    if is_falsey(self.peek(0)) {
                        ip_offset = offset as isize;
                    }
                }
                Op::Jump { offset } => {
                    ip_offset = offset as isize;
                }
                Op::Loop { offset } => {
                    ip_offset = -(offset as isize);
                }
            }
            self.current_frame_mut().ip =
                (self.current_frame().ip as isize + op_size as isize + ip_offset as isize) as usize;
        }
    }

    fn push(&mut self, value: Value) {
        if cfg!(feature = "trace") {
            println!("Pushing {value}");
        }
        self.current_frame_mut().slots.push(value);
    }

    fn pop(&mut self) -> Value {
        match self.current_frame_mut().slots.pop() {
            Some(x) => x,
            None => panic!("Tried to pop an empty stack!"),
        }
    }

    fn peek(&self, distance: usize) -> &Value {
        &self.current_frame().slots[self.current_frame().slots.len() - 1 - distance]
    }

    fn iter(&self) -> Iter<'_, Value> {
        self.current_frame().slots.iter()
    }

    fn runtime_error(&mut self, mut message: String) -> InterpretError {
        let line = self.current_frame().function.chunk.get_line_no(
            self.current_frame()
                .function
                .chunk
                .get_op_idx(cmp::max(self.current_frame().ip, 1) - 1),
        );
        writeln!(&mut message, "\n[line {line}] in script").unwrap();
        InterpretError::RuntimeError(message)
    }

    fn current_frame(&self) -> &CallFrame {
        self.frames.last().unwrap()
    }

    fn current_frame_mut(&mut self) -> &mut CallFrame {
        self.frames.last_mut().unwrap()
    }

    fn current_chunk(&mut self) -> &Chunk {
        &self.current_frame().function.chunk
    }
}

#[derive(Debug, PartialEq)]
pub enum InterpretError {
    CompileError,
    RuntimeError(String),
}

fn is_falsey(value: &Value) -> bool {
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

struct CallFrame {
    function: Function,
    ip: usize,
    slots: Vec<Value>,
}

impl CallFrame {
    fn new(function: Function, slots: Vec<Value>) -> Self {
        CallFrame {
            function,
            ip: 0,
            slots,
        }
    }
}
