use std::convert::TryFrom;
use std::convert::TryInto;
use std::rc::Rc;

use crate::object::Object;
use crate::value::Value;

mod debug;

#[derive(Debug)]
pub enum Op {
    Return,
    Constant { value: Value },
    Nil,
    False,
    True,
    Negate,
    Add,
    Subtract,
    Multiply,
    Divide,
    Not,
    Equal,
    Greater,
    Less,
    Print,
    Pop,
    DefineGlobal { name: Rc<String> },
    GetGlobal { name: Rc<String> },
    SetGlobal { name: Rc<String> },
    GetLocal { idx: u8 },
    SetLocal { idx: u8 },
    JumpIfFalse { offset: u16 },
    Jump { offset: u16 },
    Loop { offset: u16 },
}

#[derive(Debug)]
enum OpCode {
    Return,
    Constant,
    ConstantLong,
    Nil,
    True,
    False,
    Negate,
    Add,
    Subtract,
    Multiply,
    Divide,
    Not,
    Equal,
    Greater,
    Less,
    Print,
    Pop,
    DefineGlobal,
    DefineGlobalLong,
    GetGlobal,
    GetGlobalLong,
    SetGlobal,
    SetGlobalLong,
    GetLocal,
    SetLocal,
    JumpIfFalse,
    Jump,
    Loop,
}

impl OpCode {
    fn code_size(&self) -> usize {
        match self {
            OpCode::Constant => 2,
            OpCode::ConstantLong => 3,
            _ => 1,
        }
    }
}

impl TryFrom<u8> for OpCode {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(OpCode::Return),
            1 => Ok(OpCode::Constant),
            2 => Ok(OpCode::ConstantLong),
            3 => Ok(OpCode::Negate),
            4 => Ok(OpCode::Add),
            5 => Ok(OpCode::Subtract),
            6 => Ok(OpCode::Multiply),
            7 => Ok(OpCode::Divide),
            8 => Ok(OpCode::Nil),
            9 => Ok(OpCode::True),
            10 => Ok(OpCode::False),
            11 => Ok(OpCode::Not),
            12 => Ok(OpCode::Equal),
            13 => Ok(OpCode::Greater),
            14 => Ok(OpCode::Less),
            15 => Ok(OpCode::Print),
            16 => Ok(OpCode::Pop),
            17 => Ok(OpCode::DefineGlobal),
            18 => Ok(OpCode::DefineGlobalLong),
            19 => Ok(OpCode::GetGlobal),
            20 => Ok(OpCode::GetGlobalLong),
            21 => Ok(OpCode::SetGlobal),
            22 => Ok(OpCode::SetGlobalLong),
            23 => Ok(OpCode::GetLocal),
            24 => Ok(OpCode::SetLocal),
            25 => Ok(OpCode::JumpIfFalse),
            26 => Ok(OpCode::Jump),
            27 => Ok(OpCode::Loop),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Chunk {
    pub code: Vec<u8>,
    pub constants: Vec<Value>,
    pub line_nos: Vec<(u32, u32)>,
}

impl Chunk {
    pub fn new() -> Chunk {
        Chunk {
            code: vec![],
            constants: vec![],
            line_nos: vec![],
        }
    }

    pub fn push_op_code(&mut self, op: Op, line_no: u32) {
        match op {
            Op::Return => self.code.push(0),
            Op::Constant { value } => self.push_constant_op(value, 1, 2),
            Op::Negate => self.code.push(3),
            Op::Add => self.code.push(4),
            Op::Subtract => self.code.push(5),
            Op::Multiply => self.code.push(6),
            Op::Divide => self.code.push(7),
            Op::Nil => self.code.push(8),
            Op::True => self.code.push(9),
            Op::False => self.code.push(10),
            Op::Not => self.code.push(11),
            Op::Equal => self.code.push(12),
            Op::Greater => self.code.push(13),
            Op::Less => self.code.push(14),
            Op::Print => self.code.push(15),
            Op::Pop => self.code.push(16),
            Op::DefineGlobal { name } => {
                self.push_constant_op(Value::Obj(Object::String { chars: name }), 17, 18)
            }
            Op::GetGlobal { name } => {
                self.push_constant_op(Value::Obj(Object::String { chars: name }), 19, 20)
            }
            Op::SetGlobal { name } => {
                self.push_constant_op(Value::Obj(Object::String { chars: name }), 21, 22)
            }
            Op::GetLocal { idx } => {
                self.code.push(23);
                self.code.push(idx);
            }
            Op::SetLocal { idx } => {
                self.code.push(24);
                self.code.push(idx);
            }
            Op::JumpIfFalse { offset } => {
                self.code.push(25);
                self.push_u16(offset);
            }
            Op::Jump { offset } => {
                self.code.push(26);
                self.push_u16(offset);
            }
            Op::Loop { offset } => {
                self.code.push(27);
                self.push_u16(offset);
            }
        }
        self.push_line_no(line_no);
    }

    fn push_constant_op(&mut self, value: Value, short_op_code: u8, long_op_code: u8) {
        self.constants.push(value);
        const U8_SIZE: usize = ::std::mem::size_of::<u8>() * 8;
        const U8_SIZE_PLUS_1: usize = ::std::mem::size_of::<u8>() * 8 + 1;
        const U16_SIZE: usize = ::std::mem::size_of::<u16>() * 8;
        let const_idx = self.constants.len() - 1;
        match const_idx {
            0..=U8_SIZE => {
                self.code.push(short_op_code);
                self.code.push(const_idx as u8);
            }
            U8_SIZE_PLUS_1..=U16_SIZE => {
                self.code.push(long_op_code);
                self.push_u16(const_idx as u16);
            }
            _ => panic!("Tried to store constant index {} as a u16", const_idx),
        }
    }

    fn push_line_no(&mut self, line_no: u32) {
        match self.line_nos.last() {
            Some((val, count)) => {
                if *val == line_no {
                    let len = self.line_nos.len();
                    self.line_nos[len - 1] = (line_no, count + 1)
                } else {
                    self.line_nos.push((line_no, 1))
                }
            }
            None => self.line_nos.push((line_no, 1)),
        }
    }

    fn push_u16(&mut self, u16: u16) {
        self.code.push(((u16 as u16) & 0xFF) as u8);
        self.code.push(((u16 as u16) >> 8) as u8);
    }

    pub fn decode(&self, idx: usize) -> (Op, usize) {
        let code_val = self.code[idx];
        let op_code: OpCode = match code_val.try_into() {
            Ok(op_code) => op_code,
            Err(()) => {
                panic!("Invalid op code {} found at index {}!", code_val, idx)
            }
        };
        match op_code {
            OpCode::Return => (Op::Return {}, 1),
            OpCode::Constant => (
                Op::Constant {
                    value: self.get_const_short(idx),
                },
                2,
            ),
            OpCode::ConstantLong => (
                Op::Constant {
                    value: self.get_const_long(idx),
                },
                3,
            ),
            OpCode::Negate => (Op::Negate, 1),
            OpCode::Add => (Op::Add, 1),
            OpCode::Subtract => (Op::Subtract, 1),
            OpCode::Multiply => (Op::Multiply, 1),
            OpCode::Divide => (Op::Divide, 1),
            OpCode::Nil => (Op::Nil, 1),
            OpCode::True => (Op::True, 1),
            OpCode::False => (Op::False, 1),
            OpCode::Not => (Op::Not, 1),
            OpCode::Equal => (Op::Equal, 1),
            OpCode::Greater => (Op::Greater, 1),
            OpCode::Less => (Op::Less, 1),
            OpCode::Print => (Op::Print, 1),
            OpCode::Pop => (Op::Pop, 1),
            OpCode::DefineGlobal => match self.get_const_short(idx) {
                Value::Obj(Object::String { chars: name }) => (Op::DefineGlobal { name }, 2),
                _ => panic!("Expected string object value!"),
            },
            OpCode::DefineGlobalLong => match self.get_const_long(idx) {
                Value::Obj(Object::String { chars: name }) => (Op::DefineGlobal { name }, 3),
                _ => panic!("Expected string object value!"),
            },
            OpCode::GetGlobal => match self.get_const_short(idx) {
                Value::Obj(Object::String { chars: name }) => (Op::GetGlobal { name }, 2),
                _ => panic!("Expected string object value!"),
            },
            OpCode::GetGlobalLong => match self.get_const_long(idx) {
                Value::Obj(Object::String { chars: name }) => (Op::GetGlobal { name }, 3),
                _ => panic!("Expected string object value!"),
            },
            OpCode::SetGlobal => match self.get_const_short(idx) {
                Value::Obj(Object::String { chars: name }) => (Op::SetGlobal { name }, 2),
                _ => panic!("Expected string object value!"),
            },
            OpCode::SetGlobalLong => match self.get_const_long(idx) {
                Value::Obj(Object::String { chars: name }) => (Op::SetGlobal { name }, 3),
                _ => panic!("Expected string object value!"),
            },
            OpCode::GetLocal => (
                Op::GetLocal {
                    idx: self.code[idx + 1],
                },
                2,
            ),
            OpCode::SetLocal => (
                Op::SetLocal {
                    idx: self.code[idx + 1],
                },
                2,
            ),
            OpCode::JumpIfFalse => (
                Op::JumpIfFalse {
                    offset: self.get_u16(idx + 1),
                },
                3,
            ),
            OpCode::Jump => (
                Op::Jump {
                    offset: self.get_u16(idx + 1),
                },
                3,
            ),
            OpCode::Loop => (
                Op::Loop {
                    offset: self.get_u16(idx + 1),
                },
                3,
            ),
        }
    }

    fn get_const_short(&self, idx: usize) -> Value {
        let const_idx = self.code[idx + 1];
        self.constants[const_idx as usize].clone()
    }

    fn get_const_long(&self, idx: usize) -> Value {
        let const_idx = self.get_u16(idx + 1);
        self.constants[const_idx as usize].clone()
    }

    fn get_u16(&self, idx: usize) -> u16 {
        let lo = (self.code[idx]) as u16;
        let hi = (self.code[idx + 1]) as u16;
        (hi << 8) + lo
    }

    pub fn get_line_no(&self, op_idx: usize) -> u32 {
        let mut instruction_count = 0;
        for (line_no, count) in &self.line_nos {
            instruction_count += count;
            if instruction_count > (op_idx as u32) {
                return *line_no;
            }
        }
        panic!(
            "Looking for line number a instruction {} but only {} \
            line numbers recorded!",
            op_idx, instruction_count
        );
    }

    pub fn get_op_idx(&self, code_idx: usize) -> usize {
        let mut i = 0;
        let mut result = 0;
        while i < code_idx {
            i += self.decode(i).1;
            result += 0
        }
        result
    }
}
