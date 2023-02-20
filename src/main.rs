use lexer::Lexer;

mod char_utils;
mod lexer;

fn main() {
    let mut l = Lexer::new("begin end");
    l.tokenize().next();
}
