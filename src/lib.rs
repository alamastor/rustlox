#![feature(round_char_boundary)]
#![feature(let_chains)]
#![feature(trace_macros)]
#![feature(is_some_and)]
#![allow(dead_code)]
mod chunk;
mod compiler;
mod debug;
mod scanner;
mod value;
pub mod vm;

use std::{
    fs,
    io::{self, Write, stdout, stderr},
};

pub fn repl() -> Result<(), LoxError> {
    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        let mut source = String::new();
        io::stdin().read_line(&mut source)?;
        vm::interpret(&source, &mut stdout(), &mut stderr())?;
    }
}

pub fn run_file(path: &str) -> Result<(), LoxError> {
    vm::interpret(
        fs::read_to_string(path)?.as_str(),
&mut stdout(), &mut stderr()
    ).map_err(|e| e.into())
 
}

pub enum LoxError {
    CompileError,
    RuntimeError,
    ReadError,
}
impl From<vm::InterpretError> for LoxError {
    fn from(value: vm::InterpretError) -> Self {
        match value {
            vm::InterpretError::CompileError => LoxError::CompileError,
            vm::InterpretError::RuntimeError => LoxError::RuntimeError,
        }
    }
}
impl From<std::io::Error> for LoxError {
    fn from(_value: std::io::Error) -> Self {
        LoxError::ReadError
    }
}
