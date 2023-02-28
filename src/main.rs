mod ast;
mod char_utils;
mod codegen;
mod lexer;

use std::env;
use std::fs;

use crate::ast::ASTBuilder;
use crate::codegen::CodeGenerator;
use crate::lexer::Lexer;

fn main() {
    let args: Vec<String> = env::args().collect();

    let file_path = &args[1];
    let content = fs::read_to_string(file_path).expect("Should have been able to read the file");

    let mut lexer = Lexer::new(content.as_str());
    let iter = lexer.tokenize();
    let mut builder = ASTBuilder::new(Box::new(iter));
    let mut cg = CodeGenerator::new();
    let asm = cg.generate(builder.parse());
    println!("{}", asm);
}
