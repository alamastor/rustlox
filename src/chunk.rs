use crate::value::Value;
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
impl Op {
    pub fn to_opcode(&self, chunk: &mut Chunk) -> OpCode {
        match self {
            Op::Return => OpCode::Return,
            Op::Constant { value } => {
                chunk.constants.push(*value);
                const U8_SIZE: usize = ::std::mem::size_of::<u8>() * 8;
                const U8_SIZE_PLUS_1: usize = ::std::mem::size_of::<u8>() * 8 + 1;
                const U16_SIZE: usize = ::std::mem::size_of::<u16>() * 8;
                let const_idx = chunk.constants.len() - 1;
                match const_idx {
                    0..=U8_SIZE => OpCode::Constant {
                        value: *value,
                        idx: (const_idx) as u8,
                    },
                    U8_SIZE_PLUS_1..=U16_SIZE => OpCode::ConstantLong {
                        value: *value,
                        idx: (const_idx) as u16,
                    },
                    _ => panic!("Tried to store constant index {} as a u16", const_idx),
                }
            }
            Op::Negate => OpCode::Negate,
            Op::Add => OpCode::Add,
            Op::Subtract => OpCode::Subtract,
            Op::Multiply => OpCode::Multiply,
            Op::Divide => OpCode::Divide,
            Op::Nil => OpCode::Nil,
            Op::True => OpCode::True,
            Op::False => OpCode::False,
            Op::Not => OpCode::Not,
            Op::Equal => OpCode::Equal,
            Op::Greater => OpCode::Greater,
            Op::Less => OpCode::Less,
        }
    }
}

#[derive(Debug)]
pub enum OpCode {
    Return,
    Constant { value: Value, idx: u8 },
    ConstantLong { value: Value, idx: u16 },
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
    // TODO: Can this be a macro?
    fn code(&self) -> u8 {
        match self {
            OpCode::Return => 0,
            OpCode::Constant { value: _, idx: _ } => 1,
            OpCode::ConstantLong { value: _, idx: _ } => 2,
            OpCode::Negate => 3,
            OpCode::Add => 4,
            OpCode::Subtract => 5,
            OpCode::Multiply => 6,
            OpCode::Divide => 7,
            OpCode::Nil => 8,
            OpCode::True => 9,
            OpCode::False => 10,
            OpCode::Not => 11,
            OpCode::Equal => 12,
            OpCode::Greater => 13,
            OpCode::Less => 14,
        }
    }

    pub fn code_size(&self) -> usize {
        match self {
            OpCode::Constant { value: _, idx: _ } => 2,
            OpCode::ConstantLong { value: _, idx: _ } => 3,
            _ => 1,
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
        let op_code = op.to_opcode(self);
        self.code.push(op_code.code());
        match op_code {
            OpCode::Constant { value: _, idx } => self.code.push(idx),
            OpCode::ConstantLong { value: _, idx } => {
                self.code.push((idx & 0xFF) as u8);
                self.code.push((idx >> 8) as u8);
            }
            _ => {}
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

    pub fn decode(&self, idx: usize) -> OpCode {
        let code = self.code[idx];
        match code {
            0 => OpCode::Return {},
            1 => {
                let const_idx = self.code[idx + 1];
                let value = self.constants[const_idx as usize];
                OpCode::Constant {
                    value,
                    idx: const_idx,
                }
            }
            2 => {
                let lo = (self.code[idx + 1]) as u16;
                let hi = (self.code[idx + 1]) as u16;
                let const_idx = (hi << 8) + lo;
                let value = self.constants[const_idx as usize];
                OpCode::ConstantLong {
                    value,
                    idx: const_idx,
                }
            }
            3 => OpCode::Negate,
            4 => OpCode::Add,
            5 => OpCode::Subtract,
            6 => OpCode::Multiply,
            7 => OpCode::Divide,
            8 => OpCode::Nil,
            9 => OpCode::True,
            10 => OpCode::False,
            11 => OpCode::Not,
            12 => OpCode::Equal,
            13 => OpCode::Greater,
            14 => OpCode::Less,
            _ => {
                panic!("Invalid op code {} found at index {}!", code, idx)
            }
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
            i += self.decode(i).code_size();
            result += 1
        }
        result
    }
}
