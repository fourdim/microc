use crate::lexer::{self, Token, TokenType};

#[derive(Debug, Clone)]
pub enum BinaryOpKind {
    Add,
    Sub,
}

#[derive(Debug, Clone)]
pub enum SyscallKind {
    Read,
    Write,
}

#[derive(Debug, Clone)]
pub struct ExprAST {
    kind: ExprKind,
}

#[derive(Debug, Clone)]
pub enum ExprKind {
    IntLiteralExprAST {
        value: i32,
    },
    VariableExprAST {
        name: Box<str>,
    },
    BinaryExprAST {
        op: BinaryOpKind,
        lhs: Box<ExprAST>,
        rhs: Box<ExprAST>,
    },
    SyscallExprAST {
        calle: SyscallKind,
        args: Vec<ExprAST>,
    },
}

pub struct ASTBuilder<I> {
    iter: I,
    current: Token,
}

impl<I: Iterator<Item = Token>> Iterator for ASTBuilder<I> {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        self.current = self.iter.next().unwrap_or(Token::eof());
        return Some(self.current.clone());
    }
}

impl<I: Iterator<Item = Token>> ASTBuilder<I> {
    pub fn new(iter: I) -> ASTBuilder<I> {
        ASTBuilder {
            iter,
            current: Token::unknown(),
        }
    }

    // <expression> -> <primary> <binary op rhs>
    pub fn parse_expression(&mut self) -> Option<Box<ExprAST>> {
        let lhs = self.parse_primary()?;
        return self.parse_bin_op_rhs(lhs);
    }

    // <binary op rhs> -> {<add op> <primary>}
    pub fn parse_bin_op_rhs(&mut self, mut lhs: Box<ExprAST>) -> Option<Box<ExprAST>> {
        loop {
            if !matches!(
                self.current.token_type,
                TokenType::OpPlus | TokenType::OpMinus
            ) {
                return Some(lhs);
            }
            let bin_op = match self.current.token_type {
                TokenType::OpPlus => BinaryOpKind::Add,
                TokenType::OpMinus => BinaryOpKind::Sub,
                _ => return None,
            };
            self.next();
            let rhs = self.parse_primary()?;
            lhs = Box::new(ExprAST {
                kind: ExprKind::BinaryExprAST {
                    op: bin_op,
                    lhs,
                    rhs,
                },
            });
        }
    }

    // <primary> -> Identifier
    pub fn parse_identifier(&mut self) -> Option<Box<ExprAST>> {
        let id = match self.current.clone().token_type {
            TokenType::Identifier { name } => name,
            _ => panic!(),
        };
        self.next();
        if self.current.token_type != TokenType::LeftParen {
            return Some(Box::new(ExprAST {
                kind: ExprKind::VariableExprAST { name: id },
            }));
        }

        // '('
        let mut args = Vec::<ExprAST>::new();
        if self.current.token_type != TokenType::RightParen {
            loop {
                let arg = self.parse_expression()?;
                args.push(*arg);
                if self.current.token_type == TokenType::RightParen {
                    break;
                }
                if self.current.token_type != TokenType::Comma {
                    panic!("Unexpected token: {:?}", self.current.token_type)
                }
                self.next();
            }
        }

        // ')'
        self.next();
        let calle = match &*id {
            "read" => SyscallKind::Read,
            "write" => SyscallKind::Write,
            _ => panic!("Unimplemented system call"),
        };
        Some(Box::new(ExprAST {
            kind: ExprKind::SyscallExprAST { calle, args },
        }))
    }

    // <primary> -> IntLiteral
    pub fn parse_int_literal(&mut self) -> Option<Box<ExprAST>> {
        let value = match self.current.token_type {
            TokenType::IntLiteral { value } => value,
            _ => panic!("Not an int literal"),
        };
        let result = Box::new(ExprAST {
            kind: ExprKind::IntLiteralExprAST { value },
        });
        self.next();
        Some(result)
    }

    // <primary> -> LeftParen <expression> RightParen
    pub fn parse_paren(&mut self) -> Option<Box<ExprAST>> {
        // eat '('
        self.next();
        let v = self.parse_expression()?;
        if self.current.token_type != TokenType::RightParen {
            panic!("Unexpected token: {:?}", self.current.token_type);
        }
        // eat ')'
        self.next();
        Some(v)
    }

    // <primary> -> Identifier
    // <primary> -> IntLiteral
    // <primary> -> LeftParen <expression> RightParen
    pub fn parse_primary(&mut self) -> Option<Box<ExprAST>> {
        println!("{:?}", self.current.token_type);
        match self.current.token_type {
            TokenType::Read => self.parse_identifier(),
            TokenType::Write => self.parse_identifier(),
            TokenType::Identifier { name: _ } => self.parse_identifier(),

            TokenType::IntLiteral { value: _ } => self.parse_int_literal(),
            TokenType::LeftParen => self.parse_paren(),
            _ => None,
        }
    }

    pub fn parse(&mut self) {
        let mut program_start = false;
        loop {
            self.next();
            match self.current.token_type {
                TokenType::ScanEof => break,
                TokenType::Begin => program_start = true,
                TokenType::End => program_start = false,
                TokenType::Semicolon => continue,
                _ => {
                    if !program_start {
                        continue;
                    }
                }
            }
            let v = self.parse_expression();
            println!("{:?}", v);
        }
    }
}

mod tests {
    use super::*;
    use lexer::Lexer;

    #[test]
    fn handle_a_plus_b() {
        let mut lexer = Lexer::new(r#"begin read(a, b); write(a + b); end"#);
        let iter = lexer.tokenize();
        let mut builder = ASTBuilder::new(Box::new(iter));
        builder.parse();
    }
}
