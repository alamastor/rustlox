use std::convert::TryFrom;
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

impl TryFrom<u8> for OpCode {
    type Error = String;

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            x if x == OpCode::Return as u8 => Ok(OpCode::Return),
            x if x == OpCode::Constant as u8 => Ok(OpCode::Constant),
            _ => Err(format!("No OpCode matched <u8> with value {}", v)),
        }
    }
}
