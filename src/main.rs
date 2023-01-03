#![feature(trace_macros)]
#![allow(dead_code)]
mod chunk;
mod compiler;
mod debug;
mod scanner;
mod vm;
use std::io::{self, Write};

fn main() {
    match repl() {
        Ok(()) => {}
        Err(err) => {
            println!(
                "{}",
                match err {
                    LoxError::CompileError => "Compile error!",
                    LoxError::RuntimeError => "Runtime error!",
                    LoxError::ReadError => "Read error!",
                }
            )
        }
    }
}

fn repl() -> Result<(), LoxError> {
    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        let mut source = String::new();
        io::stdin().read_line(&mut source)?;
        vm::interpret(&source)?;
    }
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
