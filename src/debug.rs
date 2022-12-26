use crate::chunk::OpCode;

pub trait Disassemble {
    fn disassemble(&self, name: &str);
}

impl Disassemble for Vec<OpCode> {
    fn disassemble(&self, name: &str) {
        println!("== {name} ==");

        for (offset, op_code) in self.iter().enumerate() {
            let formatted_op_code = format_op_code(op_code);
            println!("{offset:04} {formatted_op_code}");
        }
    }
}

fn format_op_code(op_code: &OpCode) -> String {
    match op_code {
        OpCode::Return => format!("OP_RETURN"),
    }
}
