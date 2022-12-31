use crate::chunk::{Chunk, OpCode, OpInfo};

impl Chunk {
    pub fn disassemble(&self, name: &str) {
        println!("== {name} ==");

        let mut prev_line_no = None;
        for OpInfo {
            op_code,
            line_no,
            code_offset,
        } in self.iter()
        {
            let line_no_str: String;
            match prev_line_no {
                Some(x) => {
                    if line_no == x {
                        line_no_str = "   |".to_owned()
                    } else {
                        line_no_str = format!("{:04}", line_no);
                    }
                }
                None => {
                    line_no_str = format!("{:4}", line_no);
                }
            }
            prev_line_no = Some(line_no);
            let formatted_op_code = self.format_op_code(op_code);
            println!("{code_offset:04} {line_no_str} {formatted_op_code}");
        }
    }

    fn format_op_code(&self, op_code: OpCode) -> String {
        match op_code {
            OpCode::Return {} => format!("OP_RETURN"),
            OpCode::Constant { value, idx } => format!("OP_CONSTANT        {idx} '{value}'"),
        }
    }
}
