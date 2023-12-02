use std::mem::size_of;

use crate::chunk::{Chunk, Op};
use crate::object::{Function, Object};
use crate::scanner::{Scanner, Token, TokenData};
use crate::strings::Strings;
use crate::value::Value;

pub fn compile(source: &str) -> Result<(Function, Vec<Object>, Strings), ()> {
    let mut parser = Parser::new(source);
    if cfg!(feature = "trace") {
        parser.current_chunk().disassemble("chunk".to_string());
    }

    while !parser.match_(Token::Eof) {
        parser.declaration();
    }
    parser.consume(Token::Eof, "Expected EOF".to_string());
    parser.end_compiler();
    if parser.had_error {
        Err(())
    } else {
        Ok((parser.compiler.function, parser.objects, parser.strings))
    }
}

struct Parser<'a> {
    scanner: Scanner<'a>,
    compiler: Compiler,
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
            compiler: Compiler::new(),
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

    fn block(&mut self) {
        while self.scanner.peek().token != Token::RightBrace
            && self.scanner.peek().token != Token::Eof
        {
            self.declaration();
        }

        self.consume(Token::RightBrace, "Expect '}' after block.".to_string());
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

    fn for_statement(&mut self) {
        self.begin_scope();
        self.consume(Token::LeftParen, "Expect '(' after 'for'.".to_string());
        if self.match_(Token::Semicolon) {
            // No initializer.
        } else if self.match_(Token::Var) {
            self.var_declaration();
        } else {
            self.expression_statement();
        }

        let mut loop_start = self.current_chunk().code.len();
        let mut exit_jump = None;
        if !self.match_(Token::Semicolon) {
            self.expression();
            self.consume(Token::Semicolon, "Expect ';'.".to_string());

            // Jump out of the loop if the condition is false.
            exit_jump = Some(self.emit_jump(Op::JumpIfFalse { offset: 0xFFFF }));
            self.emit_byte(Op::Pop);
        }

        if !self.match_(Token::RightParen) {
            let body_jump = self.emit_jump(Op::Jump { offset: 0xFFFF });
            let increment_start = self.current_chunk().code.len();
            self.expression();
            self.emit_byte(Op::Pop);
            self.consume(
                Token::RightParen,
                "Expect ')' after for clauses.".to_string(),
            );

            self.emit_loop(loop_start);
            loop_start = increment_start;
            self.patch_jump(body_jump);
        }

        self.statement();
        self.emit_loop(loop_start);

        if let Some(exit_jump) = exit_jump {
            self.patch_jump(exit_jump);
            self.emit_byte(Op::Pop);
        }
        self.end_scope();
    }

    fn if_statement(&mut self) {
        self.consume(Token::LeftParen, "Expect '(' after 'if'.".to_string());
        self.expression();
        self.consume(Token::RightParen, "Expect ')' after condition.".to_string());

        let then_jump = self.emit_jump(Op::JumpIfFalse { offset: 0xFFFF });
        self.emit_byte(Op::Pop);
        self.statement();

        let else_jump = self.emit_jump(Op::Jump { offset: 0xFFFF });

        self.patch_jump(then_jump);
        self.emit_byte(Op::Pop);

        if self.match_(Token::Else) {
            self.statement()
        }
        self.patch_jump(else_jump);
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
        } else if self.match_(Token::For) {
            self.for_statement();
        } else if self.match_(Token::If) {
            self.if_statement();
        } else if self.match_(Token::While) {
            self.while_statement();
        } else if self.match_(Token::LeftBrace) {
            self.begin_scope();
            self.block();
            self.end_scope();
        } else {
            self.expression_statement();
        }
    }

    fn print_statement(&mut self) {
        self.expression();
        self.consume(Token::Semicolon, "Expect ';' after value.".to_string());
        self.emit_byte(Op::Print);
    }

    fn while_statement(&mut self) {
        let loop_start = self.current_chunk().code.len();

        self.consume(Token::LeftParen, "Expect '(' after 'while'.".to_string());
        self.expression();
        self.consume(Token::RightParen, "Expect ')' after 'while'.".to_string());

        let exit_jump = self.emit_jump(Op::JumpIfFalse { offset: 0xFFFF });
        self.emit_byte(Op::Pop);
        self.statement();
        self.emit_loop(loop_start);

        self.patch_jump(exit_jump);
        self.emit_byte(Op::Pop);
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

    fn or(&mut self) {
        let else_jump = self.emit_jump(Op::JumpIfFalse { offset: 0xFFFF });
        let end_jump = self.emit_jump(Op::Jump { offset: 0xFFFF });

        self.patch_jump(else_jump);
        self.emit_byte(Op::Pop);

        self.parse_precedence(Precedence::Or as usize);
        self.patch_jump(end_jump);
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
        let object = Object::String {
            chars: self.strings.new_string(string_data),
        };
        self.objects.push(object);
        self.emit_constant(Value::Obj(self.objects.last().unwrap().clone()));
    }

    fn variable(&mut self, can_assign: bool) {
        self.named_variable(self.prev_token, can_assign);
    }

    fn named_variable(&mut self, name: TokenData, can_assign: bool) {
        let get_op;
        let set_op;

        if let Some(idx) = self.resolve_local(&name.token) {
            get_op = Op::GetLocal { idx };
            set_op = Op::SetLocal { idx };
        } else {
            let arg = self.identifier_constant(name);
            let name = self.strings.new_string(arg);
            get_op = Op::GetGlobal { name: name.clone() };
            set_op = Op::SetGlobal { name: name.clone() };
        }

        if can_assign && self.match_(Token::Equal) {
            self.expression();
            self.emit_byte(set_op);
        } else {
            self.emit_byte(get_op);
        }
    }

    fn resolve_local(&mut self, name: &Token) -> Option<u8> {
        for (i, local) in self.compiler.locals.iter().enumerate().rev() {
            if &local.name == name {
                if local.depth == None {
                    self.error("Can't read local variable in its own initializer.".to_string());
                }
                return Some(i as u8);
            }
        }
        return None;
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
        let line = self.prev_token.line;
        self.current_chunk().push_op_code(op, line)
    }

    fn emit_bytes(&mut self, op_1: Op, op_2: Op) {
        self.emit_byte(op_1);
        self.emit_byte(op_2);
    }

    fn emit_loop(&mut self, loop_start: usize) {
        let offset = self.current_chunk().code.len() - loop_start + 3;
        if offset > u16::MAX as usize {
            self.error("Loop body too large.".to_string());
        }
        self.emit_byte(Op::Loop {
            offset: offset as u16,
        })
    }

    fn emit_jump(&mut self, op: Op) -> usize {
        self.emit_byte(op);
        return self.current_chunk().code.len() - 2;
    }

    fn emit_constant(&mut self, value: Value) {
        self.emit_byte(Op::Constant { value });
    }

    fn patch_jump(&mut self, offset: usize) {
        // -2 to adjust for the bytecode for the jump offset itself.
        let jump = self.current_chunk().code.len() - offset - 2;

        if jump > u16::MAX as usize {
            self.error("Too much code to jump over.".to_string());
        }
        self.current_chunk().code[offset] = (jump & 0xFF) as u8;
        self.current_chunk().code[offset + 1] = (jump >> 8) as u8;
    }

    fn end_compiler(&mut self) {
        self.emit_byte(Op::Return);
        if cfg!(feature = "trace") {
            let name = self.compiler.function.name.clone();
            let name = name.unwrap_or("<script>".to_string());
            self.current_chunk().disassemble(name);
        }
    }

    fn begin_scope(&mut self) {
        self.compiler.scope_depth += 1;
    }

    fn end_scope(&mut self) {
        self.compiler.scope_depth -= 1;

        while let Some(local) = self.compiler.locals.last()
            && let Some(depth) = local.depth && depth > self.compiler.scope_depth
        {
            self.emit_byte(Op::Pop);
            self.compiler.locals.pop();
        }
    }

    fn parse_precedence(&mut self, precedence: usize) {
        self.advance();

        let can_assign = precedence <= Precedence::Assignment as usize;
        match self.prev_token.token {
            Token::LeftParen => self.grouping(),
            Token::Minus => self.unary(),
            Token::Number => self.number(),
            Token::False => self.literal(),
            Token::True => self.literal(),
            Token::Nil => self.literal(),
            Token::Bang => self.unary(),
            Token::String => self.string(),
            Token::Identifier => self.variable(can_assign),
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
                Token::And => self.and(),
                Token::Or => self.or(),
                _ => self.error("Expect expression".to_string()),
            }
        }

        if can_assign && self.match_(Token::Equal) {
            self.error("Invalid assignment target.".to_string());
        }
    }

    fn parse_variable(&mut self, error_message: String) -> String {
        self.consume(Token::Identifier, error_message);

        self.declare_variable();
        if self.compiler.scope_depth > 0 {
            return "".to_string();
        }

        self.identifier_constant(self.prev_token)
    }

    fn mark_initialized(&mut self) {
        self.compiler.locals.last_mut().unwrap().depth = Some(self.compiler.scope_depth);
    }

    fn define_variable(&mut self, name: String) {
        if self.compiler.scope_depth > 0 {
            self.mark_initialized();
            return;
        }

        let string = self.strings.new_string(name);
        self.emit_byte(Op::DefineGlobal { name: string })
    }

    fn and(&mut self) {
        let end_jump = self.emit_jump(Op::JumpIfFalse { offset: 0xFFFF });

        self.emit_byte(Op::Pop);
        self.parse_precedence(Precedence::And as usize);

        self.patch_jump(end_jump);
    }

    fn identifier_constant(&mut self, token_data: TokenData) -> String {
        token_data.source.to_string()
    }

    fn add_local(&mut self, name: Token) {
        let local = Local { name, depth: None };
        if self.compiler.locals.len() >= size_of::<u8>() {
            self.error("Too many locals".to_string());
        } else {
            self.compiler.locals.push(local);
        }
    }

    fn declare_variable(&mut self) {
        if self.compiler.scope_depth == 0 {
            return;
        }

        let name = self.prev_token.token;
        for local in self.compiler.locals.iter().rev() {
            if let Some(depth) = local.depth && depth < self.compiler.scope_depth {
                break;
            }

            if name == local.name {
                self.error("Already a variable with this name in this scope.".to_string());
                break;
            }
        }

        self.add_local(name);
    }

    fn current_precedence(&mut self) -> Precedence {
        self.scanner.peek().token.get_precedence()
    }

    fn current_chunk(&mut self) -> &mut Chunk {
        &mut self.compiler.function.chunk
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
            Token::And => Precedence::And,
            Token::Or => Precedence::Or,
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

struct Compiler {
    locals: Vec<Local>,
    scope_depth: usize,
    function: Function,
    function_type: FunctionType,
}

impl Compiler {
    fn new() -> Self {
        Compiler {
            locals: vec![],
            scope_depth: 0,
            function: Function::new(None),
            function_type: FunctionType::Script,
        }
    }
}

struct Local {
    name: Token,
    depth: Option<usize>,
}

enum FunctionType {
    Function,
    Script,
}
