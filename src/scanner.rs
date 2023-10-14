use rstest::*;
macro_rules! identifier_chars {
    () => {'_' | 'a'..='z' | 'A'..='Z'};
}

pub struct Scanner<'a> {
    source: &'a str,
    idx: usize,
    line: u32,
}
impl<'a> Scanner<'a> {
    pub fn new(source: &str) -> Scanner {
        Scanner {
            source,
            idx: 0,
            line: 1,
        }
    }

    fn make_token_data(&self, token: Token) -> TokenData<'a> {
        self.make_token_data_with_start(token, self.idx - 1)
    }

    fn make_token_data_with_start(&self, token: Token, start: usize) -> TokenData<'a> {
        TokenData {
            token,
            line: self.line,
            source: &self.source[start..self.source.ceil_char_boundary(self.idx)],
            start,
        }
    }

    fn match_char(&mut self, c: char) -> bool {
        self.match_condition(|ch| ch == c)
    }

    fn match_condition<F>(&mut self, condition: F) -> bool
    where
        F: Fn(char) -> bool,
    {
        match self.peek_char() {
            Some(c) => {
                if condition(c) {
                    self.next_char();
                    true
                } else {
                    false
                }
            }
            None => false,
        }
    }

    fn string(&mut self) -> TokenData<'a> {
        let start = self.idx - 1;
        loop {
            match self.next_char() {
                Some(c) => {
                    if c == '"' {
                        return self.make_token_data_with_start(Token::String, start);
                    }
                }
                None => {
                    return self.make_token_data_with_start(
                        Token::Error(ErrorToken::UnterminatedString),
                        start,
                    );
                }
            }
        }
    }

    fn number(&mut self) -> TokenData<'a> {
        let start = self.idx - 1;
        loop {
            match self.peek_char() {
                Some('.') => {
                    self.next_char();
                    loop {
                        match self.peek_char() {
                            Some('0'..='9') => {
                                self.next_char();
                            }
                            _ => {
                                return self.make_token_data_with_start(Token::Number, start);
                            }
                        }
                    }
                }
                Some('0'..='9') => {
                    self.next_char();
                }
                _ => {
                    return self.make_token_data_with_start(Token::Number, start);
                }
            }
        }
    }

    fn identifier(&mut self) -> TokenData<'a> {
        let start = self.idx - 1;
        let mut len = 1;
        let token = match loop {
            let c = self.peek_char();
            if let Some(identifier_chars!()) = c {
                self.next_char();
                len += 1;
            } else {
                break &self.source[start..start + len];
            }
        } {
            "and" => Token::And,
            "class" => Token::Class,
            "else" => Token::Else,
            "false" => Token::False,
            "for" => Token::For,
            "fun" => Token::Fun,
            "if" => Token::If,
            "nil" => Token::Nil,
            "or" => Token::Or,
            "print" => Token::Print,
            "return" => Token::Return,
            "super" => Token::Super,
            "true" => Token::True,
            "this" => Token::This,
            "var" => Token::Var,
            "while" => Token::While,
            _ => Token::Identifier,
        };

        self.make_token_data_with_start(token, start)
    }

    fn next_char(&mut self) -> Option<char> {
        if self.idx >= self.source.len() {
            return None;
        }
        let result = self.source[self.idx..self.source.ceil_char_boundary(self.idx + 1)]
            .chars()
            .next();
        self.idx = self.source.ceil_char_boundary(self.idx + 1);
        result
    }

    fn peek_char(&self) -> Option<char> {
        if self.idx >= self.source.len() {
            return None;
        }
        self.source[self.idx..self.source.ceil_char_boundary(self.idx + 1)]
            .chars()
            .next()
    }

    pub fn next(&mut self) -> TokenData<'a> {
        let next_char = self.next_char();
        self.char_to_token_data(next_char)
    }

    pub fn peek(&mut self) -> TokenData<'a> {
        let saved_idx = self.idx;
        let next_char = self.next_char();
        let result = self.char_to_token_data(next_char);
        self.idx = saved_idx;
        result
    }

    fn char_to_token_data(&mut self, char: Option<char>) -> TokenData<'a> {
        match char {
            Some(ch) => match ch {
                '(' => self.make_token_data(Token::LeftParen),
                ')' => self.make_token_data(Token::RightParen),
                '{' => self.make_token_data(Token::LeftBrace),
                '}' => self.make_token_data(Token::RightBrace),
                ';' => self.make_token_data(Token::Semicolon),
                ',' => self.make_token_data(Token::Comma),
                '.' => self.make_token_data(Token::Dot),
                '-' => self.make_token_data(Token::Minus),
                '+' => self.make_token_data(Token::Plus),
                '/' => {
                    if self.match_char('/') {
                        while self.match_condition(|c| c != '\n') {}
                        self.next()
                    } else {
                        self.make_token_data(Token::Slash)
                    }
                }
                '*' => self.make_token_data(Token::Star),
                '!' => {
                    let start = self.idx;
                    if self.match_char('=') {
                        self.make_token_data_with_start(Token::BangEqual, start)
                    } else {
                        self.make_token_data(Token::Bang)
                    }
                }
                '=' => {
                    let start = self.idx;
                    if self.match_char('=') {
                        self.make_token_data_with_start(Token::EqualEqual, start)
                    } else {
                        self.make_token_data(Token::Equal)
                    }
                }
                '<' => {
                    let start = self.idx;
                    if self.match_char('=') {
                        self.make_token_data_with_start(Token::LessEqual, start)
                    } else {
                        self.make_token_data(Token::Less)
                    }
                }
                '>' => {
                    if self.match_char('=') {
                        self.make_token_data(Token::GreaterEqual)
                    } else {
                        self.make_token_data(Token::Greater)
                    }
                }
                ' ' | '\r' | '\t' => self.next(),
                '\n' => {
                    self.line += 1;
                    self.next()
                }
                '"' => self.string(),
                '0'..='9' => self.number(),
                identifier_chars!() => self.identifier(),
                _ => self.make_token_data(Token::Error(ErrorToken::InvalidToken(ch))),
            },
            None => TokenData {
                token: Token::Eof,
                line: self.line,
                source: "",
                start: self.idx,
            },
        }
    }
}

#[rstest]
#[case("1", vec![TokenData {token: Token::Number, source: "1", start: 0, line: 1}])]
#[case("1 2 \n", vec![
    TokenData {token: Token::Number, source: "1", start: 0, line: 1},
    TokenData {token: Token::Number, source: "2", start: 2, line: 1},
    TokenData {token: Token::Eof, source: "", start: 5, line: 2},
])]
#[case("true", vec![TokenData {token: Token::True, source: "true", start: 0, line: 1}])]
#[case("1 + 2\n", vec![
    TokenData {token: Token::Number, source: "1", start: 0, line: 1},
    TokenData {token: Token::Plus, source: "+", start: 2, line: 1},
    TokenData {token: Token::Number, source: "2", start: 4, line: 1},
    TokenData {token: Token::Eof, source: "", start: 6, line: 2},
])]
#[case("  \"two\"  \"strings\" \n", vec![
    TokenData {token: Token::String, source: "\"two\"", start: 2, line: 1},
    TokenData {token: Token::String, source: "\"strings\"", start: 9, line: 1},
    TokenData {token: Token::Eof, source: "", start: 20, line: 2}])]
fn scanner(#[case] source: &str, #[case] expected_tokens: Vec<TokenData>) {
    let mut scanner = Scanner::new(source);

    for expected_token in expected_tokens {
        assert_eq!(scanner.next(), expected_token);
    }
    assert_eq!(scanner.next().token, Token::Eof);
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum Token {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    // one or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    // literals.
    Identifier,
    String,
    Number,
    // keywords.
    And,
    Class,
    Else,
    False,
    For,
    Fun,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
    // Error
    Error(ErrorToken),
    // Start of file
    Sof,
    // End of file
    Eof,
}
impl Token {
    pub fn is_error(&self) -> bool {
        matches!(self, Token::Error(_))
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum ErrorToken {
    UnterminatedString,
    InvalidToken(char),
}
impl ErrorToken {
    pub fn as_string(&self) -> String {
        match self {
            ErrorToken::UnterminatedString => "Unterminated string".to_string(),
            ErrorToken::InvalidToken(char) => format!("Invalid token {char}"),
        }
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct TokenData<'a> {
    pub token: Token,
    pub line: u32,
    pub source: &'a str,
    pub start: usize,
}
