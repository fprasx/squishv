use riscv::parse_v2::Lexer;
fn main() {
    let lexer = Lexer::new("addi x0, sp, 10 beqz");
    for token in lexer {
        println!("{token:?}")
    }
}
