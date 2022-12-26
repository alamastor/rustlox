use crate::chunk::{Chunk, OpCode};

impl Chunk {
    pub fn disassemble(&self, name: &str) {
        println!("== {name} ==");

        for (offset, op_code) in self.code.iter().enumerate() {
            let formatted_op_code = self.format_op_code(op_code);
            println!("{offset:04} {formatted_op_code}");
        }
    }

    fn format_op_code(&self, op_code: &OpCode) -> String {
        match op_code {
            OpCode::Return => format!("OP_RETURN"),
            OpCode::Constant(idx) => {
                let val = self.constants[*idx];
                format!("OP_CONST {val}")
            }
        }
    }
}
