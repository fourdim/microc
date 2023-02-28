use crate::ast::{ExprAST, ExprKind};
use std::collections::BTreeMap;

pub static PRELUDE: &'static str = r#"# Module : main
    .text
    .globl read
read:
    # call read integer
    li $v0 5
    syscall
    jr $ra
    .data
data_section_$$1:
    .word '\n'
    .text
    .globl write
write:
    lw $t0, data_section_$$1
    li $v0, 1
    syscall
    move $a0, $t0
    li $v0, 11
    syscall
    jr $ra
"#;

// main function prologue
pub fn main_prologue(stackframe_size: u32) -> String {
    format!(
        "
    .text
    .globl main
main:
    # prologue area
    addi $sp, $sp, -{}
    sw $ra, 20($sp)
    sw $fp, 28($sp)
    move $fp, $sp
",
        stackframe_size
    )
}

// main function epilogue
pub fn main_epilogue(stackframe_size: u32) -> String {
    format!(
        "
    # epilogue area
    move $sp, $fp
    lw $fp, 28($sp)
    lw $ra, 20($sp)
    addi $sp, $sp, {}
    li $v0 10
    syscall
",
        stackframe_size
    )
}

#[derive(Debug)]
pub struct CodeGenerator {
    pub frame_size: u32,
    pub frame_pointer: u32,
    pub symbol_map: BTreeMap<String, u32>,
    pub asm: Vec<String>,
}

pub enum Operand {
    MEM(u32),
    IMM(i32),
}

impl CodeGenerator {
    pub fn new() -> CodeGenerator {
        CodeGenerator {
            frame_size: 32,
            frame_pointer: 32,
            symbol_map: BTreeMap::new(),
            asm: Vec::new(),
        }
    }

    pub fn generate(&mut self, statements: Vec<Box<ExprAST>>) -> String {
        let mut buf = String::new();

        for expr in statements.into_iter() {
            self.generate_functions(expr);
        }

        buf.push_str(main_prologue(self.frame_pointer).as_str());
        for c in self.asm.iter() {
            buf.push_str("    ");
            buf.push_str(c.as_str());
            buf.push('\n');
        }
        buf.push_str(main_epilogue(self.frame_pointer).as_str());
        buf.push_str(PRELUDE);
        buf
    }

    // gen write, read and assignment function
    pub fn generate_functions(&mut self, expr: Box<ExprAST>) {
        match expr.kind {
            ExprKind::SyscallExprAST { calle, args } => match calle {
                crate::ast::SyscallKind::Read => {
                    for e in args.into_iter() {
                        if let Operand::MEM(offset) = self.codegen(Box::new(e)) {
                            self.asm.push("jal read".to_string());
                            self.asm.push(format!("sw $v0, {}($fp)", offset));
                        }
                    }
                }
                crate::ast::SyscallKind::Write => {
                    for e in args.into_iter() {
                        match self.codegen(Box::new(e)) {
                            Operand::MEM(offset) => {
                                self.asm.push(format!("lw $a0, {}($fp)", offset))
                            }
                            Operand::IMM(imm) => self.asm.push(format!("li $a0, {}", imm)),
                        }
                        self.asm.push("jal write".to_string());
                    }
                }
            },
            ExprKind::AssignmentAST { var, assign } => {
                let mut left_side = 0;
                if let Operand::MEM(offset) = self.codegen(var) {
                    left_side = offset;
                }
                match self.codegen(assign) {
                    Operand::MEM(offset) => self.asm.push(format!("lw $t0, {}($fp)", offset)),
                    Operand::IMM(imm) => self.asm.push(format!("li $t0, {}", imm)),
                }
                self.asm.push(format!("sw $t0, {}($fp)", left_side));
            }
            _ => panic!(),
        }
    }

    // recursive parse expression AST
    pub fn codegen(&mut self, expr: Box<ExprAST>) -> Operand {
        match expr.kind {
            ExprKind::VariableExprAST { name } => Operand::MEM(
                self.symbol_map
                    .entry(name.to_string())
                    .or_insert_with(|| {
                        self.frame_pointer += 4;
                        self.frame_pointer - 4
                    })
                    .clone(),
            ),
            ExprKind::IntLiteralExprAST { value } => Operand::IMM(value.clone()),
            ExprKind::BinaryExprAST { op, lhs, rhs } => {
                let left_hand_side = self.codegen(lhs);
                let right_hand_side = self.codegen(rhs);

                match left_hand_side {
                    Operand::MEM(offset) => self.asm.push(format!("lw $t0, {}($fp)", offset)),
                    Operand::IMM(imm) => self.asm.push(format!("li $t0, {}", imm)),
                }
                match right_hand_side {
                    Operand::MEM(offset) => self.asm.push(format!("lw $t1, {}($fp)", offset)),
                    Operand::IMM(imm) => self.asm.push(format!("li $t1, {}", imm)),
                }

                match op {
                    crate::ast::BinaryOpKind::Add => self.asm.push("add $t0, $t0, $t1".to_string()),
                    crate::ast::BinaryOpKind::Sub => self.asm.push("sub $t0, $t0, $t1".to_string()),
                }

                self.asm
                    .push(format!("sw $t0, {}($fp)", self.frame_pointer));
                self.frame_pointer += 4;
                Operand::MEM(self.frame_pointer - 4)
            }
            _ => panic!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::ASTBuilder;
    use crate::lexer::Lexer;

    use super::*;

    #[test]
    fn handle_a_plus_b() {
        let mut lexer = Lexer::new(
            r#"-- Input: 1, Expected Output: 1326
            begin
            
            read (A0);  
            A1 := A0 + 1;  
            A2 := A1 + 1;  
            A3 := A2 + 1;  
            A4 := A3 + 1;  
            A5 := A4 + 1;  
            A6 := A5 + 1;  
            A7 := A6 + 1;  
            A8 := A7 + 1;  
            A9 := A8 + 1;  
            A10 := A9 + 1;  
            A11 := A10 + 1;  
            A12 := A11 + 1;  
            A13 := A12 + 1;  
            A14 := A13 + 1;  
            A15 := A14 + 1;  
            A16 := A15 + 1;  
            A17 := A16 + 1;  
            A18 := A17 + 1;  
            A19 := A18 + 1;  
            A20 := A19 + 1;  
            A21 := A20 + 1;  
            A22 := A21 + 1;  
            A23 := A22 + 1;  
            A24 := A23 + 1;  
            A25 := A24 + 1;  
            A26 := A25 + 1;  
            A27 := A26 + 1;  
            A28 := A27 + 1;  
            A29 := A28 + 1;  
            A30 := A29 + 1;  
            A31 := A30 + 1;  
            A32 := A31 + 1;  
            A33 := A32 + 1;  
            A34 := A33 + 1;  
            A35 := A34 + 1;  
            A36 := A35 + 1;  
            A37 := A36 + 1;  
            A38 := A37 + 1;  
            A39 := A38 + 1;  
            A40 := A39 + 1;  
            A41 := A40 + 1;  
            A42 := A41 + 1;  
            A43 := A42 + 1;  
            A44 := A43 + 1;  
            A45 := A44 + 1;  
            A46 := A45 + 1;  
            A47 := A46 + 1;  
            A48 := A47 + 1;  
            A49 := A48 + 1;  
            A50 := A49 + 1;  
            write( A0+A1+A2+A3+A4+A5+A6+A7+A8+A9+A10+A11+A12+A13+A14+A15+A16+A17+A18+A19+A20+A21+A22+A23+A24+A25+A26+A27+A28+A29+A30+A31+A32+
            A33+A34+A35+A36+A37+A38+A39+A40+A41+A42+A43+A44+A45+A46+A47+A48+A49+A50);
            
            end"#,
        );
        let iter = lexer.tokenize();
        let mut builder = ASTBuilder::new(Box::new(iter));
        let mut cg = CodeGenerator::new();
        let asm = cg.generate(builder.parse());
        println!("{}", asm);
    }
}
