use riscv::parse::Program;

fn main() -> anyhow::Result<()> {
    let s = format!("{:?}", "f".parse::<Program>().unwrap_err());
    println!("{s}");
    Ok(())
}
