use std::str::Chars;

use crate::char_utils;

const EOF_CHAR: char = '\0';

#[derive(PartialEq, Debug)]
pub enum TokenType {
    Whitespace,
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
    LineComment,
    Unknown,
    ScanEof,
}

impl TokenType {
    fn as_str(&self) -> &'static str {
        match self {
            TokenType::Whitespace => "Whitespace",
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
            TokenType::LineComment => "LineComment",
            TokenType::Unknown => "Unknown",
            TokenType::ScanEof => "ScanEof",
        }
    }
}

#[derive(Debug)]
pub struct Token {
    token_type: TokenType,
    length: u32,
    line: usize,
    column: usize,
}

impl Token {
    pub fn new(lexer: &Lexer) -> Token {
        Token {
            token_type: TokenType::Unknown,
            length: 0,
            line: lexer.line,
            column: lexer.column,
        }
    }

    pub fn set_type(&mut self, token_type: TokenType) {
        self.token_type = token_type
    }

    pub fn set_length(&mut self, len: u32) {
        self.length = len;
    }
}

pub struct Lexer<'a> {
    len_remaining: usize,
    source: &'a str,
    /// Iterator over chars. Slightly faster than a &str.
    chars: Chars<'a>,
    line: usize,
    column: usize,
    offset: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(content: &'a str) -> Lexer<'a> {
        Lexer {
            len_remaining: content.len(),
            source: content,
            chars: content.chars(),
            line: 1,
            column: 1,
            offset: 0,
        }
    }

    /// Peeks the next symbol from the input stream without consuming it.
    /// If requested position doesn't exist, `EOF_CHAR` is returned.
    /// However, getting `EOF_CHAR` doesn't always mean actual end of file,
    /// it should be checked with `is_eof` method.
    fn first(&self) -> char {
        // `.next()` optimizes better than `.nth(0)`
        self.chars.clone().next().unwrap_or(EOF_CHAR)
    }

    /// Peeks the second symbol from the input stream without consuming it.
    fn second(&self) -> char {
        // `.next()` optimizes better than `.nth(1)`
        let mut iter = self.chars.clone();
        iter.next();
        iter.next().unwrap_or(EOF_CHAR)
    }

    /// Checks if there is nothing more to consume.
    fn is_eof(&self) -> bool {
        self.chars.as_str().is_empty()
    }

    /// Returns amount of already consumed symbols.
    fn token_length(&self) -> u32 {
        (self.len_remaining - self.chars.as_str().len()) as u32
    }

    /// Resets the number of bytes consumed to 0.
    fn reset_token_length(&mut self) {
        self.len_remaining = self.chars.as_str().len();
    }

    /// Moves to the next character.
    fn bump(&mut self) -> Option<char> {
        let c = self.chars.next()?;
        self.offset += 1;
        if c == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }

        Some(c)
    }

    /// Eats symbols while predicate returns true or until the end of file is reached.
    fn eat_while(&mut self, mut predicate: impl FnMut(char) -> bool) {
        // It was tried making optimized version of this for eg. line comments, but
        // LLVM can inline all of this and compile it down to fast iteration over bytes.
        while predicate(self.first()) && !self.is_eof() {
            self.bump();
        }
    }

    /// Eats symbols until predicate returns true or until the end of file is reached.
    fn eat_until(&mut self, mut predicate: impl FnMut(char) -> bool) {
        // It was tried making optimized version of this for eg. line comments, but
        // LLVM can inline all of this and compile it down to fast iteration over bytes.
        while !predicate(self.first()) && !self.is_eof() {
            self.bump();
        }
    }

    pub fn next_token(&mut self) -> Token {
        let mut token = Token::new(&self);

        let first_char = match self.bump() {
            Some(c) => c,
            None => {
                token.set_type(TokenType::ScanEof);
                return token;
            }
        };

        let token_type = match first_char {
            // OpMinus or line comment.
            '-' => match self.first() {
                '-' => {
                    self.bump();
                    self.eat_while(|c| c != '\n');
                    TokenType::LineComment
                }
                _ => TokenType::OpMinus,
            },
            c if c.is_whitespace() => {
                self.eat_while(char_utils::is_whitespace);
                TokenType::Whitespace
            }
            c if c.is_ascii_alphabetic() => {
                self.eat_while(char_utils::is_identifier_continue);
                // At here keywords are also identified as identifiers.
                TokenType::Identifier
            }
            '0'..='9' => {
                self.eat_while(char_utils::is_digit);
                TokenType::IntLiteral
            }
            '(' => TokenType::LeftParen,
            ')' => TokenType::RightParen,
            ';' => TokenType::Semicolon,
            ',' => TokenType::Comma,
            '=' => TokenType::OpAssign,
            '+' => TokenType::OpPlus,
            // Only compile when `usize` is larger or equal to 32 bit.
            _ => {
                self.eat_until(char_utils::is_expected);
                self.syntax_error("unexpected char(s)")
            }
        };
        token.set_type(token_type);
        token.set_length(self.token_length());
        self.reset_token_length();
        token
    }

    /// Creates an iterator that produces tokens from the input string.
    pub fn tokenize(&'a mut self) -> impl Iterator<Item = Token> + '_ {
        std::iter::from_fn(move || {
            let token = self.next_token();
            if token.token_type != TokenType::ScanEof {
                Some(token)
            } else {
                None
            }
        })
    }
}

impl Lexer<'_> {
    // fn new_int_literal(&mut self, scanned_string: String, mut token: Token) -> Token {
    //     self.column += scanned_string.len();
    //     token.set_type(TokenType::IntLiteral);
    //     token.set_inner_string(scanned_string);
    //     return token;
    // }

    // fn new_identifier_or_keyword(&mut self, mut token: Token, scanned_string: String) -> Token {
    //     self.column += scanned_string.len();

    //     macro_rules! check_keyword {
    //         ($x:expr) => {
    //             if scanned_string == $x.as_str() {
    //                 token.set_type($x);
    //                 return token;
    //             }
    //         };
    //     }

    //     check_keyword!(TokenType::Begin);
    //     check_keyword!(TokenType::End);
    //     check_keyword!(TokenType::Read);
    //     check_keyword!(TokenType::Write);

    //     token.set_type(TokenType::Identifier);
    //     token.set_inner_string(scanned_string);
    //     token
    // }

    fn syntax_error(&mut self, msg: &str) -> ! {
        let len = self.token_length() as usize;
        let lines: Vec<&str> = self.source.lines().collect();
        let width = self.column - 1;
        panic!(
            r#"microc: [syntax error] {}
    --> {}:{}
      |
{:>5} |{}
      |{:>width$}
"#,
            msg,
            self.line,
            self.column - len,
            self.line,
            lines[self.line - 1],
            "^".repeat(len)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{stderr, Write};

    impl Lexer<'_> {
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
                let token = self.next_token();
                if token.token_type == TokenType::ScanEof {
                    let mut count = 3;
                    while count > 0 {
                        assert_eq!(self.next_token().token_type, TokenType::ScanEof);
                        count -= 1;
                    }
                    break;
                }
            }
        }

        fn print_token_list(&mut self) {
            let mut count = 30;
            loop {
                let token = self.next_token();

                println!("{:?}", token);

                if count < 0 {
                    break;
                } else {
                    count -= 1;
                }

                if token.token_type == TokenType::ScanEof {
                    let mut count = 3;
                    while count > 0 {
                        assert_eq!(self.next_token().token_type, TokenType::ScanEof);
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


        "#,
        );
        lexer.test_loop();
    }

    #[test]
    #[should_panic]
    fn handle_error_keyword() {
        let mut lexer = Lexer::new(
            r#"

        1EGIN END


        "#,
        );
        lexer.test_loop();
    }

    #[test]
    #[should_panic]
    fn handle_nonexist_char() {
        let mut lexer = Lexer::new(
            r#"
        BEGIN ****()
        END
        "#,
        );
        lexer.test_loop();
    }

    #[test]
    fn handle_a_plus_b() {
        let mut lexer = Lexer::new(r#"BEGIN READ(a, b); WRITE(a + b); END"#);
        lexer.print_token_list();
    }
}
