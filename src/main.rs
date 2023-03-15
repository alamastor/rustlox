use lox::{repl, run_file, LoxError};

use std::process;

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
