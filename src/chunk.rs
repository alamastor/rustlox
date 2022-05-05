pub struct Chunk {
    pub code: Vec<OpCode>,
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

    pub fn write(&mut self, code: OpCode, line: u64) {
        self.code.push(code);
        self.lines.push(line);
    }

    pub fn add_constant(&mut self, value: f64) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }
}

pub enum OpCode {
    Return,
    Constant(usize),
}
