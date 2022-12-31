pub enum Op {
    Return,
    Constant { value: f64 },
}
impl Op {
    pub fn to_opcode(&self, chunk: &mut Chunk) -> OpCode {
        match self {
            Op::Return => OpCode::Return,
            Op::Constant { value } => {
                chunk.constants.push(*value);
                const U8_SIZE: usize = ::std::mem::size_of::<u8>();
                const U16_SIZE: usize = ::std::mem::size_of::<u16>();
                let const_idx = chunk.constants.len() - 1;
                match const_idx {
                    0..=U8_SIZE => OpCode::Constant {
                        value: *value,
                        idx: (const_idx) as u8,
                    },
                    ..=U16_SIZE => OpCode::ConstantLong {
                        value: *value,
                        idx: (const_idx) as u16,
                    },
                    _ => panic!("Tried to store constant index {} as a u16", const_idx),
                }
            }
        }
    }
}

pub enum OpCode {
    Return,
    Constant { value: f64, idx: u8 },
    ConstantLong { value: f64, idx: u16 },
}
impl OpCode {
    // TODO: Can this be a macro?
    fn code(&self) -> u8 {
        match self {
            OpCode::Return => 0,
            OpCode::Constant { value: _, idx: _ } => 1,
            OpCode::ConstantLong { value: _, idx: _ } => 2,
        }
    }

    fn code_size(&self) -> usize {
        match self {
            OpCode::Return => 1,
            OpCode::Constant { value: _, idx: _ } => 2,
            OpCode::ConstantLong { value: _, idx: _ } => 3,
        }
    }
}

pub struct Chunk {
    pub code: Vec<u8>,
    pub constants: Vec<f64>,
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
            OpCode::Return => {}
            OpCode::Constant { value: _, idx } => self.code.push(idx),
            OpCode::ConstantLong { value: _, idx } => {
                self.code.push((idx & 0xFF) as u8);
                self.code.push((idx >> 8) as u8);
            }
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

    pub fn iter(&self) -> ChunkIterator {
        ChunkIterator {
            chunk: self,
            idx: 0,
            code_idx: 0,
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
            _ => {
                panic!("Invalid op code {} found at index {}!", code, idx)
            }
        }
    }

    fn get_line_no(&self, idx: usize) -> u32 {
        let mut instruction_count = 0;
        for (line_no, count) in &self.line_nos {
            instruction_count += count;
            if instruction_count > (idx as u32) {
                return *line_no;
            }
        }
        panic!(
            "Looking for line number a instruction {} but only {} \
            line numbers recorded!",
            idx, instruction_count
        );
    }
}

pub struct ChunkIterator<'a> {
    chunk: &'a Chunk,
    idx: usize,
    code_idx: usize,
}

impl<'a> Iterator for ChunkIterator<'a> {
    type Item = OpInfo;

    fn next(&mut self) -> Option<Self::Item> {
        if self.code_idx < self.chunk.code.len() {
            let op_code = self.chunk.decode(self.code_idx);
            let line_no = self.chunk.get_line_no(self.idx);
            let result = OpInfo {
                op_code,
                line_no,
                code_offset: self.code_idx,
            };
            self.idx += 1;
            self.code_idx += result.op_code.code_size();
            Some(result)
        } else {
            None
        }
    }
}

pub struct OpInfo {
    pub op_code: OpCode,
    pub line_no: u32,
    pub code_offset: usize,
}
