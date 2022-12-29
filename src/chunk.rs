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
}

impl Chunk {
    pub fn new() -> Chunk {
        Chunk {
            code: vec![],
            constants: vec![],
        }
    }

    pub fn push_op_code(&mut self, op_code: Op) {
        self.code.push(op_code.code());
        match op_code {
            Op::Return {} => {}
            Op::Constant { value, extras: _ } => {
                self.constants.push(value);
                self.code
                    .push((self.constants.len() - 1).try_into().unwrap())
            }
        }
    }

    pub fn iter(&self) -> ChunkIterator {
        ChunkIterator {
            chunk: self,
            idx: 0,
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
}

pub struct ChunkIterator<'a> {
    chunk: &'a Chunk,
    idx: usize,
}

impl<'a> Iterator for ChunkIterator<'a> {
    type Item = Op;

    fn next(&mut self) -> Option<Self::Item> {
        self.chunk.decode(self.idx).map(|op| {
            self.idx += op.code_size();
            op
        })
    }
}
