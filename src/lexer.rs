#[derive(PartialEq, Debug)]
pub enum TokenType {
    Begin,
    End,
    Read,
    Write,
    Identifier,
    IntLiteral,
    LeftParen,
    RightParen,
    Semicolon,
    Comma,
    OpAssign,
    OpPlus,
    OpMinus,
    ScanEof,
}

impl TokenType {
    fn as_str(&self) -> &'static str {
        match self {
            TokenType::Begin => "BEGIN",
            TokenType::Comma => ",",
            TokenType::End => "END",
            TokenType::Read => "READ",
            TokenType::Write => "WRITE",
            TokenType::Identifier => "Identifier",
            TokenType::IntLiteral => "IntLiteral",
            TokenType::LeftParen => "(",
            TokenType::RightParen => ")",
            TokenType::Semicolon => ";",
            TokenType::OpAssign => "=",
            TokenType::OpPlus => "+",
            TokenType::OpMinus => "-",
            TokenType::ScanEof => "ScanEof",
        }
    }

    fn get_by_char(ch: char) -> Option<TokenType> {
        match ch {
            ',' => Some(TokenType::Comma),
            '(' => Some(TokenType::LeftParen),
            ')' => Some(TokenType::RightParen),
            ';' => Some(TokenType::Semicolon),
            '=' => Some(TokenType::OpAssign),
            '+' => Some(TokenType::OpPlus),
            '-' => Some(TokenType::OpMinus),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct Token {
    token_type: TokenType,
    inner_string: String,
    line: usize,
    column: usize,
}

impl Token {
    pub fn new(lexer: &Lexer) -> Token {
        Token {
            token_type: TokenType::ScanEof,
            inner_string: String::from(""),
            line: lexer.line,
            column: lexer.column,
        }
    }

    pub fn set_type(&mut self, token_type: TokenType) {
        self.token_type = token_type
    }

    pub fn set_inner_string(&mut self, inner_string: String) {
        self.inner_string = inner_string;
    }
}

pub struct Lexer {
    source: String,
    line: usize,
    column: usize,
    offset: usize,
    last_char: char,
}

impl Lexer {
    pub fn new(content: String) -> Lexer {
        Lexer {
            source: content,
            line: 0,
            column: 0,
            offset: 0,
            last_char: ' ',
        }
    }

    fn get_cursor(&mut self) -> Option<char> {
        let result = self.source.chars().nth(self.offset);
        self.offset += 1;
        result
    }

    fn get_next_token(&mut self) -> Token {
        while self.last_char.is_whitespace() {
            if self.last_char != '\n' {
                if self.offset != 0 {
                    self.column += 1;
                }
            } else {
                self.column = 0;
                self.line += 1;
            }
            self.last_char = match self.get_cursor() {
                Some(ch) => ch,
                None => return Token::new(self),
            };
        }

        let mut token = Token::new(self);

        if self.last_char.is_alphabetic() {
            let mut scanned_string = String::from("");

            scanned_string.push(self.last_char);

            self.last_char = match self.get_cursor() {
                Some(ch) => ch,
                None => {
                    self.last_char = ' ';
                    return self.new_identifier_or_keyword(token, scanned_string);
                }
            };

            while self.last_char.is_alphanumeric() {
                scanned_string.push(self.last_char);
                self.last_char = match self.get_cursor() {
                    Some(ch) => ch,
                    None => {
                        self.last_char = ' ';
                        return self.new_identifier_or_keyword(token, scanned_string);
                    }
                };
            }

            return self.new_identifier_or_keyword(token, scanned_string);
        }

        if self.last_char.is_ascii_digit() {
            let mut scanned_string = String::from("");

            while self.last_char.is_ascii_digit() {
                scanned_string.push(self.last_char);
                self.last_char = match self.get_cursor() {
                    Some(ch) => ch,
                    None => {
                        self.last_char = ' ';
                        return self.new_int_literal(scanned_string, token);
                    }
                };
            }

            if self.last_char.is_whitespace() {
                return self.new_int_literal(scanned_string, token);
            }

            while !self.last_char.is_whitespace() {
                scanned_string.push(self.last_char);
                self.last_char = match self.get_cursor() {
                    Some(ch) => ch,
                    None => self.syntax_error(scanned_string.len()),
                };
            }

            self.syntax_error(scanned_string.len())
        }

        match TokenType::get_by_char(self.last_char) {
            Some(token_type) => {
                self.column += 1;
                token.set_type(token_type);
                self.last_char = match self.get_cursor() {
                    Some(ch) => ch,
                    None => ' ',
                };
                token
            }
            None => self.syntax_error(1),
        }
    }

    fn new_int_literal(&mut self, scanned_string: String, mut token: Token) -> Token {
        self.column += scanned_string.len();
        token.set_type(TokenType::IntLiteral);
        token.set_inner_string(scanned_string);
        return token;
    }

    fn new_identifier_or_keyword(&mut self, mut token: Token, scanned_string: String) -> Token {
        self.column += scanned_string.len();

        macro_rules! check_keyword {
            ($x:expr) => {
                if scanned_string == $x.as_str() {
                    token.set_type($x);
                    return token;
                }
            };
        }

        check_keyword!(TokenType::Begin);
        check_keyword!(TokenType::End);
        check_keyword!(TokenType::Read);
        check_keyword!(TokenType::Write);

        token.set_type(TokenType::Identifier);
        token.set_inner_string(scanned_string);
        token
    }

    fn syntax_error(&mut self, len: usize) -> ! {
        let lines: Vec<&str> = self.source.lines().collect();
        let width = len + self.column;
        panic!(
            r#"microc: syntax error
    --> {}:{}
      |
{:>5} |{}
      |{:>width$}
"#,
            self.line + 1,
            self.column + 1,
            self.line + 1,
            lines[self.line],
            "^".repeat(len)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{stderr, Write};

    impl Lexer {
        fn test_loop(&mut self) {
            std::panic::set_hook(Box::new(|info| {
                let msg = match info.payload().downcast_ref::<&'static str>() {
                    Some(s) => *s,
                    None => match info.payload().downcast_ref::<String>() {
                        Some(s) => &s[..],
                        None => "Box<dyn Any>",
                    },
                };
                let mut lock = stderr().lock();
                writeln!(lock, "{msg}").unwrap();
            }));
            loop {
                let token = self.get_next_token();
                if token.token_type == TokenType::ScanEof {
                    let mut count = 3;
                    while count > 0 {
                        assert_eq!(self.get_next_token().token_type, TokenType::ScanEof);
                        count -= 1;
                    }
                    break;
                }
            }
        }

        fn print_token_list(&mut self) {
            let mut count = 30;
            loop {
                let token = self.get_next_token();

                println!("{:?}", token);

                if count < 0 {
                    break;
                } else {
                    count -= 1;
                }

                if token.token_type == TokenType::ScanEof {
                    let mut count = 3;
                    while count > 0 {
                        assert_eq!(self.get_next_token().token_type, TokenType::ScanEof);
                        count -= 1;
                    }
                    break;
                }
            }
        }
    }

    #[test]
    fn handle_white_space() {
        let mut lexer = Lexer::new(
            r#"

        BEGIN END


        "#
            .to_string(),
        );
        lexer.test_loop();
    }

    #[test]
    #[should_panic]
    fn handle_error_keyword() {
        let mut lexer = Lexer::new(
            r#"

        1EGIN END


        "#
            .to_string(),
        );
        lexer.test_loop();
    }

    #[test]
    fn handle_a_plus_b() {
        let mut lexer = Lexer::new(r#"BEGIN READ(a, b); WRITE(a + b); END"#.to_string());
        lexer.print_token_list();
    }
}
