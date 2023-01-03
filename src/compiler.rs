use crate::scanner::Scanner;

pub fn compile(source: &str) {
    let mut line = None;
    for token in Scanner::new(source) {
        match token {
            Ok(token_data) => {
                match line {
                    Some(l) => {
                        if l != token_data.line {
                            print!("{:04} ", token_data.line);
                        } else {
                            print!("   | ");
                        }
                    }
                    None => print!("{:04} ", token_data.line),
                };
                println!(
                    "{:?} {} {}",
                    token_data.token, token_data.start, token_data.length
                );
                line = Some(token_data.line);
            }
            Err(err) => println!("{err}"),
        }
    }
}
