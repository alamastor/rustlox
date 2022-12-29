use crate::chunk::{Chunk, Op};

impl Chunk {
    pub fn disassemble(&self, name: &str) {
        println!("== {name} ==");

        for (offset, op_code) in self.iter().enumerate() {
            let formatted_op_code = self.format_op_code(op_code);
            println!("{offset:04} {formatted_op_code}");
        }
    }

    fn format_op_code(&self, op_code: Op) -> String {
        match op_code {
            Op::Return {} => format!("OP_RETURN"),
            Op::Constant { value, extras } => {
                format!("OP_CONST {value}  '{}'", extras.unwrap().index)
            }
        }
    }
}
