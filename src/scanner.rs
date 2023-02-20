macro_rules! identifier_chars {
    () => {'_' | 'a'..='z' | 'A'..='Z'};
}

pub struct Scanner<'a> {
    source: &'a str,
    chars: ::std::iter::Peekable<::std::str::CharIndices<'a>>,
    idx: usize,
    line: u32,
}
impl<'a> Scanner<'a> {
    pub fn new(source: &str) -> Scanner {
        Scanner {
            source,
            chars: source.char_indices().peekable(),
            idx: 0,
            line: 1,
        }
    }

    fn make_token_data(&self, token: Token) -> Option<TokenData<'a>> {
        self.make_token_data_with_start(token, self.idx)
    }

    fn make_token_data_with_start(&self, token: Token, start: usize) -> Option<TokenData<'a>> {
        Some(TokenData {
            token,
            line: self.line,
            source: &self.source[start..self.source.ceil_char_boundary(self.idx + 1)],
            start,
        })
    }

    fn match_char(&mut self, c: char) -> bool {
        self.match_condition(|ch| ch == c)
    }

    fn match_condition<F>(&mut self, condition: F) -> bool
    where
        F: Fn(char) -> bool,
    {
        match self.chars.peek() {
            Some((_, c)) => {
                if condition(*c) {
                    self.next_char();
                    true
                } else {
                    false
                }
            }
            None => false,
        }
    }

    fn string(&mut self) -> Option<TokenData<'a>> {
        let start = self.idx;
        loop {
            match self.next_char() {
                Some(c) => match c {
                    '"' => {
                        return self.make_token_data_with_start(Token::String, start);
                    }
                    _ => {}
                },
                None => {
                    return self.make_token_data_with_start(
                        Token::Error(ErrorToken::UnterminatedString),
                        start,
                    );
                }
            }
        }
    }

    fn number(&mut self) -> Option<TokenData<'a>> {
        let start = self.idx;
        loop {
            match self.chars.peek() {
                Some((_, '.')) => {
                    self.next_char();
                    loop {
                        match self.chars.peek() {
                            Some((_, '0'..='9')) => {
                                self.next_char();
                            }
                            _ => {
                                return self.make_token_data_with_start(Token::Number, start);
                            }
                        }
                    }
                }
                Some((_, '0'..='9')) => {
                    self.next_char();
                }
                _ => {
                    return self.make_token_data_with_start(Token::Number, start);
                }
            }
        }
    }

    fn identifier(&mut self) -> Option<TokenData<'a>> {
        let start = self.idx;
        let mut len = 1;
        let token = match loop {
            let c = self.chars.peek();
            if let Some((_, identifier_chars!())) = c {
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
        self.chars.next().map(|(i, ch)| {
            self.idx = i;
            ch
        })
    }
}

impl<'a> Iterator for Scanner<'a> {
    type Item = TokenData<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_char()
            .map(|ch| match ch {
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
            })
            .flatten()
    }
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
}
impl Token {
    pub fn is_error(&self) -> bool {
        match self {
            Token::Error(_) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum ErrorToken {
    UnterminatedString,
    InvalidToken(char),
}
impl ErrorToken {
    pub fn to_string(&self) -> String {
        match self {
            ErrorToken::UnterminatedString => "Unterminated string".to_string(),
            ErrorToken::InvalidToken(char) => format!("Invalid token {}", char),
        }
    }
}

#[derive(PartialEq)]
pub struct TokenData<'a> {
    pub token: Token,
    pub line: u32,
    pub source: &'a str,
    pub start: usize,
}
