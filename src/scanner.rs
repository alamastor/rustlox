macro_rules! identifier_chars {
    () => {'_' | 'a'..='z' | 'A'..='Z'};
}

pub struct Scanner<'a> {
    source: &'a str,
    chars: ::std::iter::Peekable<::std::str::CharIndices<'a>>,
    line: u32,
}
impl<'a> Scanner<'a> {
    pub fn new(source: &str) -> Scanner {
        Scanner {
            source,
            chars: source.char_indices().peekable(),
            line: 1,
        }
    }

    fn make_token_data(
        &self,
        token: Token,
        char_len: usize,
        start: usize,
    ) -> Option<Result<TokenData<'a>, String>> {
        Some(Ok(TokenData {
            token,
            line: self.line,
            source: &self.source[start..start + char_len],
        }))
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
                    self.chars.next();
                    true
                } else {
                    false
                }
            }
            None => false,
        }
    }

    fn string(&mut self, start: usize) -> Option<Result<TokenData<'a>, String>> {
        let mut len = 1;
        loop {
            match self.chars.next() {
                Some((_, c)) => match c {
                    '"' => {
                        return self.make_token_data(Token::String, len + 1, start);
                    }
                    _ => {
                        len += 1;
                    }
                },
                None => {
                    return Some(Err("Unterminated string!".to_string()));
                }
            }
        }
    }

    fn number(&mut self, start: usize) -> Option<Result<TokenData<'a>, String>> {
        let mut len = 1;
        loop {
            match self.chars.peek() {
                Some((_, '.')) => {
                    self.chars.next();
                    len += 1;
                    loop {
                        match self.chars.peek() {
                            Some((_, '0'..='9')) => {
                                self.chars.next();
                                len += 1;
                            }
                            Some((_, _)) => {
                                return self.make_token_data(Token::Number, len, start);
                            }
                            None => {
                                return self.make_token_data(Token::Number, len, start);
                            }
                        }
                    }
                }
                Some((_, '0'..='9')) => {
                    self.chars.next();
                    len += 1
                }
                Some((_, _)) => {
                    return self.make_token_data(Token::Number, len, start);
                }
                None => {
                    return self.make_token_data(Token::Number, len, start);
                }
            }
        }
    }

    fn identifier(&mut self, start: usize) -> Option<Result<TokenData<'a>, String>> {
        let mut len = 1;
        let token = match loop {
            let c = self.chars.peek();
            if let Some((_, identifier_chars!())) = c {
                self.chars.next();
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

        self.make_token_data(token, len, start)
    }
}

impl<'a> Iterator for Scanner<'a> {
    type Item = Result<TokenData<'a>, String>;

    fn next(&mut self) -> Option<Self::Item> {
        self.chars
            .next()
            .map(|(i, ch)| match ch {
                '(' => self.make_token_data(Token::LeftParen, 1, i),
                ')' => self.make_token_data(Token::RightParen, 1, i),
                '{' => self.make_token_data(Token::LeftBrace, 1, i),
                '}' => self.make_token_data(Token::RightBrace, 1, i),
                ';' => self.make_token_data(Token::Semicolon, 1, i),
                ',' => self.make_token_data(Token::Comma, 1, i),
                '.' => self.make_token_data(Token::Dot, 1, i),
                '-' => self.make_token_data(Token::Minus, 1, i),
                '+' => self.make_token_data(Token::Plus, 1, i),
                '/' => {
                    if self.match_char('/') {
                        while self.match_condition(|c| c != '\n') {}
                        self.next()
                    } else {
                        self.make_token_data(Token::Slash, 1, i)
                    }
                }
                '*' => self.make_token_data(Token::Star, 1, i),
                '!' => {
                    if self.match_char('=') {
                        self.make_token_data(Token::BangEqual, 2, i)
                    } else {
                        self.make_token_data(Token::Bang, 1, i)
                    }
                }
                '=' => {
                    if self.match_char('=') {
                        self.make_token_data(Token::EqualEqual, 2, i)
                    } else {
                        self.make_token_data(Token::Equal, 1, i)
                    }
                }
                '<' => {
                    if self.match_char('=') {
                        self.make_token_data(Token::LessEqual, 2, i)
                    } else {
                        self.make_token_data(Token::Less, 1, i)
                    }
                }
                '>' => {
                    if self.match_char('=') {
                        self.make_token_data(Token::GreaterEqual, 2, i)
                    } else {
                        self.make_token_data(Token::Greater, 1, i)
                    }
                }
                ' ' | '\r' | '\t' => self.next(),
                '\n' => {
                    self.line += 1;
                    self.next()
                }
                '"' => self.string(i),
                '0'..='9' => self.number(i),
                identifier_chars!() => self.identifier(i),
                _ => Some(Err(format!("Invalid token: '{ch}'"))),
            })
            .flatten()
    }
}

#[derive(Debug)]
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
}

pub struct TokenData<'a> {
    pub token: Token,
    pub line: u32,
    pub source: &'a str,
}
