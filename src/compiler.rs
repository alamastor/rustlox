use crate::scanner::Scanner;

pub fn compile(source: &str) {
    let mut line = None;
    for token_data in Scanner::new(source) {
        if let  Some(l) = line && l == token_data.line {
            print!("   | ");
        } else {
            print!("{:04} ", token_data.line);
        };
        println!("{:?} {}", token_data.token, token_data.source);
        line = Some(token_data.line);
    }
}
