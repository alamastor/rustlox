use crate::chunk::{Chunk, Op};
use crate::scanner::{Scanner, Token, TokenData};
use crate::value::Value;
pub fn compile(source: &str) -> Result<Chunk, ()> {
    let mut parser = Parser::new(source);

    parser.expression();
    parser.consume(None, "Expected EOF".to_string());
    parser.end_compiler();
    if parser.had_error {
        Err(())
    } else {
        Ok(parser.chunk)
    }
}

struct Parser<'a> {
    scanner: ::std::iter::Peekable<Scanner<'a>>,
    chunk: Chunk,
    prev_token: Option<TokenData<'a>>,
    had_error: bool,
    panic_mode: bool,
}

impl<'a> Parser<'a> {
    fn new(source: &'a str) -> Parser<'a> {
        Parser {
            scanner: Scanner::new(source).peekable(),
            chunk: Chunk::new(),
            prev_token: None,
            had_error: false,
            panic_mode: false,
        }
    }

    fn advance(&mut self) {
        self.prev_token = self.scanner.next();

        loop {
            let token_data = self.scanner.peek();
            match token_data {
                Some(td) => match &td.token {
                    Token::Error(error_type) => {
                        self.had_error = true;
                        self.panic_mode = true;
                        error_at(token_data, error_type.as_string());
                        self.scanner.next();
                    }
                    _ => {
                        break;
                    }
                },
                None => {
                    break;
                }
            }
        }
    }

    fn error(&mut self, message: String) {
        self.had_error = true;
        self.panic_mode = true;
        error_at(self.prev_token.as_ref(), message)
    }

    fn error_at_current(&mut self, message: String) {
        self.had_error = true;
        self.panic_mode = true;
        let token_data = self.scanner.peek();
        error_at(token_data, message)
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment as usize);
    }

    fn grouping(&mut self) {
        self.expression();
        self.consume(
            Some(Token::RightParen),
            "Expect ')' after expression.".to_string(),
        );
    }

    fn unary(&mut self) {
        let op_type = self.prev_token.as_ref().unwrap().token;
        self.expression();
        match op_type {
            Token::Minus => {
                self.emit_byte(Op::Negate);
            }
            Token::Bang => {
                self.emit_byte(Op::Not)
            }
            _ => panic!("Unexpected token: {:?}!", op_type),
        }
    }

    fn binary(&mut self) {
        let op_type = self.prev_token.as_ref().unwrap().token;
        let precedence = op_type.get_precedence();
        self.parse_precedence((precedence as usize) + 1);
        match op_type {
            Token::Plus => self.emit_byte(Op::Add),
            Token::Minus => self.emit_byte(Op::Subtract),
            Token::Star => self.emit_byte(Op::Multiply),
            Token::Slash => self.emit_byte(Op::Divide),
            _ => panic!(
                "Unexpected token: {:?}!",
                self.prev_token.as_ref().unwrap().token
            ),
        }
    }

    fn number(&mut self) {
        let value = self
            .prev_token
            .as_ref()
            .unwrap()
            .source
            .parse::<f64>()
            .unwrap();
        self.emit_constant(Value::Number(value));
    }

    fn literal(&mut self) {
        match self.prev_token.as_ref().unwrap().token {
            Token::False => {
                self.emit_byte(Op::False);
            }
            Token::Nil => {
                self.emit_byte(Op::Nil);
            }
            Token::True => {
                self.emit_byte(Op::True);
            }
            unexpected => {panic!("Expected a literal token; got {:?}", unexpected);}
        }
    }

    fn consume(&mut self, expected_token: Option<Token>, message: String) {
        if self.scanner.peek().map(|td| &td.token) == expected_token.as_ref() {
            self.advance();
        } else {
            self.error_at_current(message)
        }
    }

    fn emit_byte(&mut self, op: Op) {
        let line = match self.prev_token.as_ref() {
            Some(token_data) => token_data.line,
            None => 1,
        };
        self.chunk.push_op_code(op, line)
    }

    fn emit_bytes(&mut self, op_1: Op, op_2: Op) {
        self.emit_byte(op_1);
        self.emit_byte(op_2);
    }

    fn emit_constant(&mut self, value: Value) {
        self.emit_byte(Op::Constant { value });
    }

    fn end_compiler(&mut self) {
        self.emit_byte(Op::Return)
    }

    fn parse_precedence(&mut self, precedence: usize) {
        self.advance();

        match self.prev_token.as_ref().unwrap().token {
            Token::LeftParen => self.grouping(),
            Token::Minus => self.unary(),
            Token::Number => self.number(),
            Token::False => self.literal(),
            Token::True => self.literal(),
            Token::Nil => self.literal(),
            Token::Bang => self.unary(),
            _ => self.error("Expect expression".to_string()),
        }

        while precedence <= (self.current_precedence() as usize) {
            self.advance();
            match self.prev_token.as_ref().unwrap().token {
                Token::Minus => self.binary(),
                Token::Plus => self.binary(),
                Token::Slash => self.binary(),
                Token::Star => self.binary(),
                _ => self.error("Expect expression".to_string()),
            }
        }
    }

    fn current_precedence(&mut self) -> Precedence {
        match self.scanner.peek() {
            Some(token_data) => token_data.token.get_precedence(),
            None => Precedence::None,
        }
    }
}

enum Precedence {
    None,
    Assignment,
    Or,
    And,
    Equality,
    Comparison,
    Term,
    Factor,
    Unary,
    Call,
    Primary,
}

struct ParseRule<'a, 'b> {
    prefix: Option<fn(&'a mut Parser<'b>)>,
    suffix: Option<fn(&'a mut Parser<'b>)>,
    precedence: Precedence,
}

impl Token {
    fn get_precedence(&self) -> Precedence {
        match self {
            Token::Slash => Precedence::Factor,
            Token::Star => Precedence::Factor,
            Token::Minus => Precedence::Term,
            Token::Plus => Precedence::Term,
            _ => Precedence::None,
        }
    }
}

fn grouping(parser: &mut Parser) {
    parser.expression();
    parser.consume(
        Some(Token::RightParen),
        "Expect ')' after expression.".to_string(),
    );
}

fn error_at(token_data: Option<&TokenData>, message: String) {
    let td = token_data.unwrap();
    eprintln!("[line {}] Error at {}: {message}", td.line, td.start);
}
