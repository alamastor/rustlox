use std::convert::TryFrom;
use std::convert::TryInto;

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
            _ => Err(()),
        }
    }
}

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
            Op::Constant { value } => {
                self.constants.push(value);
                const U8_SIZE: usize = ::std::mem::size_of::<u8>() * 8;
                const U8_SIZE_PLUS_1: usize = ::std::mem::size_of::<u8>() * 8 + 1;
                const U16_SIZE: usize = ::std::mem::size_of::<u16>() * 8;
                let const_idx = self.constants.len() - 1;
                match const_idx {
                    0..=U8_SIZE => {
                        self.code.push(1);
                        self.code.push(const_idx as u8);
                    }
                    U8_SIZE_PLUS_1..=U16_SIZE => {
                        self.code.push(2);
                        self.code.push(((const_idx as u16) & 0xFF) as u8);
                        self.code.push(((const_idx as u16) >> 8) as u8);
                    }
                    _ => panic!("Tried to store constant index {} as a u16", const_idx),
                }
            }
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
        }
        self.push_line_no(line_no);
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
            OpCode::Constant => {
                let const_idx = self.code[idx + 1];
                let value = self.constants[const_idx as usize].clone();
                (Op::Constant { value }, 2)
            }
            OpCode::ConstantLong => {
                let lo = (self.code[idx + 1]) as u16;
                let hi = (self.code[idx + 1]) as u16;
                let const_idx = (hi << 8) + lo;
                let value = self.constants[const_idx as usize].clone();
                (Op::Constant { value }, 3)
            }
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
        }
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
            result += 1
        }
        result
    }
}
