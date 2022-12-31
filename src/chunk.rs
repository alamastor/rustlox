use std::convert::TryInto;

pub struct ConstantExtras {
    pub index: usize,
}
pub enum Op {
    Return {},
    Constant {
        value: f64,
        extras: Option<ConstantExtras>,
    },
}

// TODO: Can this be a macro?
impl Op {
    fn code(&self) -> u8 {
        match self {
            Op::Return {} => 0,
            Op::Constant {
                value: _,
                extras: _,
            } => 1,
        }
    }

    fn code_size(&self) -> usize {
        match self {
            Op::Return {} => 1,
            Op::Constant {
                value: _,
                extras: _,
            } => 2,
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

    pub fn push_op_code(&mut self, op_code: Op, line_no: u32) {
        self.code.push(op_code.code());
        match op_code {
            Op::Return {} => {}
            Op::Constant { value, extras: _ } => {
                self.constants.push(value);
                self.code
                    .push((self.constants.len() - 1).try_into().unwrap())
            }
        }
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

    fn decode(&self, idx: usize) -> Option<Op> {
        self.code
            .get(idx)
            .map(|x| match x {
                0 => Some(Op::Return {}),
                1 => {
                    let value = self.constants[(self.code[idx + 1] as usize)];
                    Some(Op::Constant {
                        value,
                        extras: Some(ConstantExtras {
                            index: self.code[(idx + 1) as usize] as usize,
                        }),
                    })
                }
                _ => None,
            })
            .flatten()
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
        self.chunk.decode(self.code_idx).map(|op| {
            let line_no = self.chunk.get_line_no(self.idx);
            let result = OpInfo {
                op,
                line_no,
                code_offset: self.code_idx,
            };
            self.idx += 1;
            self.code_idx += result.op.code_size();
            result
        })
    }
}

pub struct OpInfo {
    pub op: Op,
    pub line_no: u32,
    pub code_offset: usize,
}
