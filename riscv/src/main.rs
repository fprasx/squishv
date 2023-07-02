use riscv::parse_v2::{parse_item, Lexer};

fn main() {
    let lexer = Lexer::new("addi x0, sp, 10 beqz");
    for token in lexer {
        println!("{token:?}")
    }
    let addi = parse_item("addi s0, t1, -0x123");
    println!("{addi:?}")
}
