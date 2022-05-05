use std::convert::From;
use std::mem::size_of;

pub struct Chunk {
    pub code: Vec<u8>,
    pub lines: Vec<u64>,
    pub constants: Vec<f64>,
}

impl Chunk {
    pub fn new() -> Chunk {
        Chunk {
            code: Vec::new(),
            lines: Vec::new(),
            constants: Vec::new(),
        }
    }

    pub fn write(&mut self, code: u8, line: u64) {
        self.code.push(code);
        self.lines.push(line);
    }

    pub fn add_constant(&mut self, value: f64) -> u8 {
        if self.constants.len() >= (size_of::<u8>()) {
            panic!("Too many constants.")
        }
        self.constants.push(value);
        (self.constants.len() - 1) as u8
    }
}

pub enum OpCode {
    Return,
    Constant,
}

impl From<u8> for OpCode {
    fn from(v: u8) -> Self {
        match v {
            x if x == OpCode::Return as u8 => OpCode::Return,
            x if x == OpCode::Constant as u8 => OpCode::Constant,
            _ => panic!("No OpCode matched <u8> with value {}", v),
        }
    }
}

impl OpCode {
    pub fn size(&self) -> usize {
        match self {
            OpCode::Return => 1,
            OpCode::Constant => 2,
        }
    }
}
