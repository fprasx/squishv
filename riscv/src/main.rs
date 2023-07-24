use indoc::indoc;
use riscv::{lex::{LexerIter, Lexer}, parse::parse_item};

fn main() {
    let source = indoc! {"
        addi x0, x0, 1 
    "};
    let mut source = LexerIter::new(Lexer::new(include_str!("test.s")));
    while let Ok(token) = parse_item(&mut source) {
        println!("{token:?}")
    }
}
