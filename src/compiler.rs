use crate::chunk::{Chunk, Op};
use crate::object::Object;
use crate::scanner::{Scanner, Token, TokenData};
use crate::strings::Strings;
use crate::value::Value;

pub fn compile(source: &str) -> Result<(Chunk, Vec<Object>, Strings), ()> {
    let mut parser = Parser::new(source);
    if cfg!(feature = "trace") {
        parser.chunk.disassemble("chunk");
    }

    while !parser.match_(Token::Eof) {
        parser.declaration();
    }
    parser.consume(Token::Eof, "Expected EOF".to_string());
    parser.end_compiler();
    if parser.had_error {
        Err(())
    } else {
        Ok((parser.chunk, parser.objects, parser.strings))
    }
}

struct Parser<'a> {
    scanner: Scanner<'a>,
    chunk: Chunk,
    prev_token: TokenData<'a>,
    objects: Vec<Object>,
    strings: Strings,
    had_error: bool,
    panic_mode: bool,
}

impl<'a> Parser<'a> {
    fn new(source: &'a str) -> Parser<'a> {
        Parser {
            scanner: Scanner::new(source),
            chunk: Chunk::new(),
            objects: vec![],
            strings: Strings::new(),
            prev_token: TokenData {
                token: Token::Sof,
                line: 0,
                source: "",
                start: 0,
            },
            had_error: false,
            panic_mode: false,
        }
    }

    fn advance(&mut self) {
        self.prev_token = self.scanner.next();
        loop {
            let token_data = self.scanner.peek();
            match token_data.token {
                Token::Error(error_type) => {
                    self.had_error = true;
                    self.panic_mode = true;
                    error_at(&token_data, error_type.as_string());
                    self.scanner.next();
                }
                _ => {
                    break;
                }
            };
        }
    }

    fn error(&mut self, message: String) {
        self.had_error = true;
        self.panic_mode = true;
        error_at(&self.prev_token, message)
    }

    fn error_at_current(&mut self, message: String) {
        self.had_error = true;
        self.panic_mode = true;
        let token_data = self.scanner.peek();
        error_at(&token_data, message)
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment as usize);
    }

    fn var_declaration(&mut self) {
        let global = self.parse_variable("Expect variable name.".to_string());

        if self.match_(Token::Equal) {
            self.expression();
        } else {
            self.emit_byte(Op::Nil);
        }
        self.consume(
            Token::Semicolon,
            "Expect ';' after variable declaration.".to_string(),
        );

        self.define_variable(global);
    }

    fn expression_statement(&mut self) {
        self.expression();
        self.consume(Token::Semicolon, "Expect ';' after expression.".to_string());
        self.emit_byte(Op::Pop)
    }

    fn declaration(&mut self) {
        if self.match_(Token::Var) {
            self.var_declaration();
        } else {
            self.statement();
        }

        if self.panic_mode {
            self.synchronize();
        }
    }

    fn statement(&mut self) {
        if self.match_(Token::Print) {
            self.print_statement();
        } else {
            self.expression_statement();
        }
    }

    fn print_statement(&mut self) {
        self.expression();
        self.consume(Token::Semicolon, "Expect ';' after value.".to_string());
        self.emit_byte(Op::Print);
    }

    fn synchronize(&mut self) {
        self.panic_mode = false;

        while self.scanner.peek().token != Token::Eof {
            if self.scanner.peek().token == Token::Semicolon {
                return;
            }
            match self.prev_token.token {
                Token::Class => {
                    return;
                }
                Token::Fun => {
                    return;
                }
                Token::Var => {
                    return;
                }
                Token::For => {
                    return;
                }
                Token::If => {
                    return;
                }
                Token::While => {
                    return;
                }
                Token::Print => {
                    return;
                }
                Token::Return => {
                    return;
                }
                _ => {}
            }
            self.advance()
        }
    }

    fn grouping(&mut self) {
        self.expression();
        self.consume(
            Token::RightParen,
            "Expect ')' after expression.".to_string(),
        );
    }

    fn unary(&mut self) {
        let op_type = self.prev_token.token;
        self.expression();
        match op_type {
            Token::Minus => {
                self.emit_byte(Op::Negate);
            }
            Token::Bang => self.emit_byte(Op::Not),
            _ => panic!("Unexpected token: {:?}!", op_type),
        }
    }

    fn binary(&mut self) {
        let op_type = self.prev_token.token;
        let precedence = op_type.get_precedence();
        self.parse_precedence((precedence as usize) + 1);
        match op_type {
            Token::Plus => self.emit_byte(Op::Add),
            Token::Minus => self.emit_byte(Op::Subtract),
            Token::Star => self.emit_byte(Op::Multiply),
            Token::Slash => self.emit_byte(Op::Divide),
            Token::BangEqual => self.emit_bytes(Op::Equal, Op::Not),
            Token::EqualEqual => self.emit_byte(Op::Equal),
            Token::Greater => self.emit_byte(Op::Greater),
            Token::GreaterEqual => self.emit_bytes(Op::Less, Op::Not),
            Token::Less => self.emit_byte(Op::Less),
            Token::LessEqual => self.emit_bytes(Op::Greater, Op::Not),
            _ => panic!("Unexpected token: {:?}!", self.prev_token.token),
        }
    }

    fn number(&mut self) {
        let value = self.prev_token.source.parse::<f64>().unwrap();
        self.emit_constant(Value::Number(value));
    }

    fn literal(&mut self) {
        match self.prev_token.token {
            Token::False => {
                self.emit_byte(Op::False);
            }
            Token::Nil => {
                self.emit_byte(Op::Nil);
            }
            Token::True => {
                self.emit_byte(Op::True);
            }
            unexpected => {
                panic!("Expected a literal token; got {:?}", unexpected);
            }
        }
    }

    fn string(&mut self) {
        let string_data = self.prev_token.source[1..self.prev_token.source.len() - 1].to_string();
        let object = Object::String {chars: self.strings.new_string(string_data)};
        self.objects.push(object);
        self.emit_constant(Value::Obj(self.objects.last().unwrap().clone()));
    }

    fn variable(&mut self) {
        self.named_variable(self.prev_token);
    }

    fn named_variable(&mut self, name: TokenData) {
        let arg = self.identifier_constant(name);
        let name = self.strings.new_string(arg);
        self.emit_byte(Op::GetGlobal { name })
    }

    fn consume(&mut self, expected_token: Token, message: String) {
        if self.scanner.peek().token == expected_token {
            self.advance();
        } else {
            self.error_at_current(message)
        }
    }

    fn match_(&mut self, token: Token) -> bool {
        if self.scanner.peek().token == token {
            self.advance();
            true
        } else {
            false
        }
    }

    fn emit_byte(&mut self, op: Op) {
        println!("emitting {op:?}");
        self.chunk.push_op_code(op, self.prev_token.line)
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

        match self.prev_token.token {
            Token::LeftParen => self.grouping(),
            Token::Minus => self.unary(),
            Token::Number => self.number(),
            Token::False => self.literal(),
            Token::True => self.literal(),
            Token::Nil => self.literal(),
            Token::Bang => self.unary(),
            Token::String => self.string(),
            Token::Identifier => self.variable(),
            _ => self.error("Expect expression".to_string()),
        }

        while precedence <= (self.current_precedence() as usize) {
            self.advance();
            match self.prev_token.token {
                Token::Minus => self.binary(),
                Token::Plus => self.binary(),
                Token::Slash => self.binary(),
                Token::Star => self.binary(),
                Token::BangEqual => self.binary(),
                Token::EqualEqual => self.binary(),
                Token::Greater => self.binary(),
                Token::GreaterEqual => self.binary(),
                Token::Less => self.binary(),
                Token::LessEqual => self.binary(),
                _ => self.error("Expect expression".to_string()),
            }
        }
    }

    fn parse_variable(&mut self, error_message: String) -> String {
        self.consume(Token::Identifier, error_message);
        self.identifier_constant(self.prev_token)
    }

    fn define_variable(&mut self, name: String) {
        let string = self.strings.new_string(name);
        self.emit_byte(Op::DefineGlobal { name: string })
    }

    fn identifier_constant(&mut self, token_data: TokenData) -> String {
        token_data.source.to_string()
    }

    fn current_precedence(&mut self) -> Precedence {
        self.scanner.peek().token.get_precedence()
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
            Token::BangEqual => Precedence::Equality,
            Token::EqualEqual => Precedence::Equality,
            Token::Greater => Precedence::Comparison,
            Token::GreaterEqual => Precedence::Comparison,
            Token::Less => Precedence::Comparison,
            Token::LessEqual => Precedence::Comparison,
            _ => Precedence::None,
        }
    }
}

fn error_at(token_data: &TokenData, message: String) {
    eprintln!(
        "[line {}] Error at {}: {message}",
        token_data.line, token_data.start
    );
}
