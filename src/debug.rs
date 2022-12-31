use crate::chunk::{Chunk, OpCode};

impl Chunk {
    pub fn disassemble(&self, name: &str) {
        println!("== {name} ==");
        let mut code_idx = 0;
        let mut op_idx = 0;
        let mut prev_line_no = None;
        while code_idx < self.code.len() {
            let op_info = self.get_op_info(code_idx, Some(op_idx));
            code_idx += op_info.op_code.code_size();
            op_idx += 1;
            let line_no = op_info.line_no;
            self.print_op_info(op_info, prev_line_no);
            prev_line_no = Some(line_no);
        }
    }

    pub fn disassemble_code(&self, code_idx: usize) {
        self.print_op_info(self.get_op_info(code_idx, None), None)
    }

    fn get_op_info(&self, code_idx: usize, op_idx: Option<usize>) -> OpInfo {
        let op_code = self.decode(code_idx);
        let op_idx_unwrap = op_idx.unwrap_or(self.get_op_idx(code_idx));
        let line_no = self.get_line_no(op_idx_unwrap);
        OpInfo {
            op_code,
            line_no,
            code_idx,
        }
    }

    fn print_op_info(
        &self,
        OpInfo {
            op_code,
            line_no,
            code_idx: op_idx,
        }: OpInfo,
        prev_line_no: Option<u32>,
    ) {
        let line_no_string = match prev_line_no {
            Some(x) => {
                if line_no == x {
                    "   |".to_owned()
                } else {
                    format!("{:04}", line_no)
                }
            }
            None => {
                format!("{:4}", line_no)
            }
        };
        print!("{op_idx:04} {line_no_string} ");
        match op_code {
            OpCode::Return {} => println!("OP_RETURN"),
            OpCode::Constant { value, idx } => println!("OP_CONSTANT        {idx} '{value}'"),
            OpCode::ConstantLong { value, idx } => println!("OP_CONSTANT_LONG   {idx} '{value}'"),
            OpCode::Negate {} => println!("OP_NEGATE"),
        }
    }
}

struct OpInfo {
    pub op_code: OpCode,
    pub line_no: u32,
    pub code_idx: usize,
}
