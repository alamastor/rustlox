#![feature(trace_macros)]
#![feature(let_chains)]
#![feature(round_char_boundary)]
#![feature(is_some_and)]
#![allow(dead_code)]
mod chunk;
mod compiler;
mod debug;
mod scanner;
mod value;
mod vm;
use std::{
    fs,
    io::{self, Write},
    process,
};

fn main() {
    let result = match ::std::env::args().len() {
        1 => repl(),
        2 => run_file(std::env::args().collect::<Vec<String>>()[1].as_str()),
        _ => {
            eprintln!("Usage: clox [path]");
            process::exit(64);
        }
    };
    match result {
        Ok(()) => {}
        Err(err) => {
            println!(
                "{}",
                match err {
                    LoxError::CompileError => "Compile error!".to_string(),
                    LoxError::RuntimeError => "Runtime error!".to_string(),
                    LoxError::ReadError => "Read error!".to_string(),
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

fn run_file(path: &str) -> Result<(), LoxError> {
    vm::interpret(fs::read_to_string(path)?.as_str()).map_err(|e| e.into())
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
