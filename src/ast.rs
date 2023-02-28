use crate::lexer::{Token, TokenType};

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
    pub kind: ExprKind,
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
    AssignmentAST {
        var: Box<ExprAST>,
        assign: Box<ExprAST>,
    },
}

pub struct ASTBuilder<I> {
    iter: I,
    current: Token,
    current_string: String,
}

impl<I: Iterator<Item = Token>> Iterator for ASTBuilder<I> {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        self.current = self.iter.next().unwrap_or(Token::eof());
        self.current_string = self.current.token_type.as_str().to_string();
        return Some(self.current.clone());
    }
}

impl<I: Iterator<Item = Token>> ASTBuilder<I> {
    pub fn new(iter: I) -> ASTBuilder<I> {
        ASTBuilder {
            iter,
            current: Token::unknown(),
            current_string: String::new(),
        }
    }

    // <expression> -> <primary> <binary op rhs>
    pub fn parse_expression(&mut self) -> Option<Box<ExprAST>> {
        let lhs = self.parse_primary().unwrap_or(Box::new(ExprAST {
            kind: ExprKind::IntLiteralExprAST { value: 0 },
        }));
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
        let calle = match self.current.clone().token_type {
            TokenType::Identifier { name } => {
                self.next();
                return Some(Box::new(ExprAST {
                    kind: ExprKind::VariableExprAST { name },
                }));
            }
            TokenType::Read => SyscallKind::Read,
            TokenType::Write => SyscallKind::Write,
            _ => panic!(),
        };
        self.next();
        self.next();

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
        match self.current.token_type {
            TokenType::Read => self.parse_identifier(),
            TokenType::Write => self.parse_identifier(),
            TokenType::Identifier { name: _ } => self.parse_identifier(),

            TokenType::IntLiteral { value: _ } => self.parse_int_literal(),
            TokenType::LeftParen => self.parse_paren(),
            _ => None,
        }
    }

    pub fn parse_assign(&mut self) -> Option<Box<ExprAST>> {
        let id = match self.current.clone().token_type {
            TokenType::Identifier { name } => Some(Box::new(ExprAST {
                kind: ExprKind::VariableExprAST { name },
            })),
            _ => panic!(),
        }?;
        self.next();
        self.next();

        Some(Box::new(ExprAST {
            kind: ExprKind::AssignmentAST {
                var: id,
                assign: self.parse_expression()?,
            },
        }))
    }

    pub fn parse(&mut self) -> Vec<Box<ExprAST>> {
        let mut p_vec = Vec::<Box<ExprAST>>::new();
        let mut program_start = false;
        loop {
            self.next();
            match self.current.token_type {
                TokenType::ScanEof => break,
                TokenType::Begin => {
                    program_start = true;
                    continue;
                }
                TokenType::End => {
                    program_start = false;
                    continue;
                }
                TokenType::Semicolon => continue,
                TokenType::LineComment => continue,
                _ => {
                    if !program_start {
                        continue;
                    }
                }
            }
            let v = match self.current.clone().token_type {
                TokenType::Identifier { name: _ } => self.parse_assign(),
                _ => self.parse_expression(),
            };
            p_vec.push(v.unwrap());
        }
        p_vec
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer::Lexer;

    use super::*;

    #[test]
    fn handle_a_plus_b() {
        let mut lexer = Lexer::new(r#"begin read(a, b); write(a + b); end"#);
        let iter = lexer.tokenize();
        let mut builder = ASTBuilder::new(Box::new(iter));
        builder.parse();
    }

    #[test]
    fn handle_assign_a() {
        let mut lexer = Lexer::new(r#"begin a := 1 + 2; end"#);
        let iter = lexer.tokenize();
        let mut builder = ASTBuilder::new(Box::new(iter));
        builder.parse();
    }
}
