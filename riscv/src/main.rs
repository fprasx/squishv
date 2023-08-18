use riscv::{
    lex::Lexer,
    parse::{Program, Register}, executor::REGISTERS,
};

fn main() -> anyhow::Result<()> {
    for reg in REGISTERS {
        println!("addi {reg}, {reg}, 4")
    }
    Ok(())
}
