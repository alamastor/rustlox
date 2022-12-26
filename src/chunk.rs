pub enum OpCode {
    Return,
    Constant(usize),
}

pub struct Chunk {
    pub code: Vec<OpCode>,
    pub constants: Vec<f64>,
}

impl Chunk {
    pub fn new() -> Chunk {
        Chunk {
            code: vec![],
            constants: vec![],
        }
    }

    pub fn add_const(&mut self, value: f64) {
        self.constants.push(value);
        self.code.push(OpCode::Constant(self.constants.len() - 1))
    }

    pub fn add_return(&mut self) {
        self.code.push(OpCode::Return)
    }
}
