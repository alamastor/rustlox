use crate::chunk::OpCode;
use std::convert::TryInto;

use super::Chunk;

impl Chunk {
    pub fn disassemble(&self, name: String) {
        println!("== {name} ==");
        let mut code_idx = 0;
        let mut op_idx = 0;
        let mut prev_line_no = None;
        while code_idx < self.code.len() {
            code_idx += self.decode(code_idx).1;
            op_idx += 1;
            self.print_op_info(code_idx, Some(op_idx), prev_line_no);
            prev_line_no = Some(self.get_line_no(op_idx));
        }
    }

    pub fn disassemble_code(&self, code_idx: usize) {
        self.print_op_info(code_idx, None, None);
    }

    fn print_op_info(&self, code_idx: usize, op_idx: Option<usize>, prev_line_no: Option<u32>) {
        let idx = op_idx.unwrap_or(self.get_op_idx(code_idx));
        let line_no = self.get_line_no(idx);
        let line_no_string = match prev_line_no {
            Some(x) => {
                if line_no == x {
                    "   |".to_owned()
                } else {
                    format!("{line_no:04}")
                }
            }
            None => {
                format!("{line_no:4}")
            }
        };
        print!("{idx:04} {line_no_string} ");
        let code_val = self.code[code_idx];
        let op_code: OpCode = match code_val.try_into() {
            Ok(op_code) => op_code,
            Err(()) => {
                panic!("Invalid op code {} found at index {}!", code_val, idx);
            }
        };
        match op_code {
            OpCode::Return => println!("OP_RETURN"),
            OpCode::Constant => {
                println!(
                    "OP_CONSTANT           {idx} '{}'",
                    self.get_const_short(idx)
                );
            }
            OpCode::ConstantLong => {
                println!("OP_CONSTANT_LONG      {idx} '{}'", self.get_const_long(idx));
            }
            OpCode::Negate => println!("OP_NEGATE"),
            OpCode::Add => println!("OP_ADD"),
            OpCode::Subtract => println!("OP_SUBTRACT"),
            OpCode::Multiply => println!("OP_MULTIPLY"),
            OpCode::Divide => println!("OP_DIVIDE"),
            OpCode::False => println!("OP_FALSE"),
            OpCode::True => println!("OP_TRUE"),
            OpCode::Nil => println!("OP_NIL"),
            OpCode::Not => println!("OP_NOT"),
            OpCode::Equal => println!("OP_EQUAL"),
            OpCode::Greater => println!("OP_GREATER"),
            OpCode::Less => println!("OP_LESS"),
            OpCode::Print => println!("OP_PRINT"),
            OpCode::Pop => println!("OP_POP"),
            OpCode::DefineGlobal => {
                println!(
                    "OP_DEFINE_GLOBAL      {idx} '{}'",
                    self.get_const_short(idx)
                );
            }
            OpCode::DefineGlobalLong => {
                println!("OP_DEFINE_GLOBAL_LONG {idx} '{}'", self.get_const_long(idx));
            }
            OpCode::GetGlobal => {
                println!(
                    "OP_GET_GLOBAL         {idx} '{}'",
                    self.get_const_short(idx)
                );
            }
            OpCode::SetGlobalLong => {
                println!("OP_SET_GLOBAL_LONG    {idx} '{}'", self.get_const_long(idx));
            }
            OpCode::SetGlobal => {
                println!(
                    "OP_SET_GLOBAL         {idx} '{}'",
                    self.get_const_short(idx)
                );
            }
            OpCode::GetGlobalLong => {
                println!("OP_GET_GLOBAL_LONG    {idx} '{}'", self.get_const_long(idx));
            }
            OpCode::GetLocal => {
                println!("OP_GET_LOCAL          {idx} '{}'", self.code[idx + 1]);
            }
            OpCode::SetLocal => {
                println!("OP_SET_LOCAL          {idx} '{}'", self.code[idx + 1]);
            }
            OpCode::JumpIfFalse => {
                println!("OP_JUMP_IF_FALSE      {idx} '{}'", self.get_u16(idx + 1));
            }
            OpCode::Jump => {
                println!("OP_JUMP               {idx} '{}'", self.get_u16(idx + 1));
            }
            OpCode::Loop => {
                println!("OP_LOOP               {idx} '{}'", self.get_u16(idx + 1));
            }
        }
    }
}
