use std::convert::From;
use std::mem::size_of;

pub struct Chunk {
    pub code: Vec<u8>,
    lines: Vec<LineCount>,
    pub constants: Vec<f64>,
}

struct LineCount {
    line: usize,
    last_byte: usize,
}

impl Chunk {
    pub fn new() -> Chunk {
        Chunk {
            code: Vec::new(),
            lines: Vec::new(),
            constants: Vec::new(),
        }
    }

    pub fn write(&mut self, code: u8, line_number: usize) {
        self.code.push(code);
        self.update_lines(line_number);
    }

    fn update_lines(&mut self, line_number: usize) {
        let lines_len = self.lines.len();
        match self.lines.last() {
            None => self.lines.push(LineCount {
                line: line_number,
                last_byte: 0,
            }),
            Some(line_count) => {
                let new_line_count = LineCount {
                    line: line_number,
                    last_byte: line_count.last_byte + 1,
                };
                if line_count.line == line_number {
                    self.lines[lines_len - 1] = new_line_count;
                } else {
                    self.lines.push(new_line_count)
                }
            }
        }
    }

    pub fn add_constant(&mut self, value: f64) -> u8 {
        if self.constants.len() >= (size_of::<u8>()) {
            panic!("Too many constants.")
        }
        self.constants.push(value);
        (self.constants.len() - 1) as u8
    }

    pub fn read_constant(&self, byte_index: usize) -> f64 {
        self.constants[self.code[byte_index] as usize]
    }

    pub fn get_line(&self, instruction_idx: usize) -> usize {
        let mut result: Option<usize> = None;
        for lc in &self.lines {
            if instruction_idx <= lc.last_byte {
                result = Some(lc.line);
            }
        }
        match result {
            Some(res) => res,
            _ => panic!(format!(
                "No line recorded for instruction with index {}",
                instruction_idx
            )),
        }
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
