use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[allow(non_camel_case_types)]
#[rustfmt::skip]
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum Register {
    x0, ra, sp, gp, tp,
    t0, t1, t2, t3, t4, t5, t6,
    a0, a1, a2, a3, a4, a5, a6, a7,
    s0, s1, s2, s3, s4, s5, s6, s7, s8, s9, s10, s11,
}

macro_rules! register_parse_impl {
    ($( $reg:ident )*) => {
        impl FromStr for Register {
            type Err = &'static str;
            fn from_str(s: &str) -> Result<Register, &'static str> {
                match s.trim() {
                    $(
                        stringify!($reg) => Ok(Register::$reg),
                    )*
                    "zero" => Ok(Register::x0),
                    _ => Err("unrecognized register")
                }
            }
        }
    }
}

register_parse_impl! {
    x0 ra sp gp tp
    t0 t1 t2 t3 t4 t5 t6
    a0 a1 a2 a3 a4 a5 a6 a7
    s0 s1 s2 s3 s4 s5 s6 s7 s8 s9 s10 s11
}

/// Represents a register and offset like `0(x0)`.
/// This is mainly implemented so it's easy to parse with the macro
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone, Copy)]
pub struct RegOffset {
    reg: Register,
    offset: i32,
}

impl FromStr for RegOffset {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (num, mut reg) = s.split_once('(').ok_or("no parentheses in token")?;
        // Trim the close parenthesis
        reg = &reg[..reg.find(')').ok_or("expected a closing parenthesis")?];
        Ok(RegOffset {
            reg: reg.parse()?,
            offset: parse_int::parse::<i32>(num.trim()).map_err(|e| e.to_string())?,
        })
    }
}

#[rustfmt::skip]
#[allow(non_camel_case_types)]
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub enum Instruction {
    // Register immediate
    addi  { rd: Register, r1: Register, imm: i32, },
    slti  { rd: Register, r1: Register, imm: i32, },
    sltiu { rd: Register, r1: Register, imm: i32, },
    xori  { rd: Register, r1: Register, imm: i32, },
    ori   { rd: Register, r1: Register, imm: i32, },
    andi  { rd: Register, r1: Register, imm: i32, },
    slli  { rd: Register, r1: Register, imm: i32, },
    srli  { rd: Register, r1: Register, imm: i32, },
    srai  { rd: Register, r1: Register, imm: i32, },

    // Loady bois
    lui { rd: Register, imm: i32, },
    li  { rd: Register, imm: i32, },

    // Register register
    add  { rd: Register, r1: Register, r2: Register, },
    sub  { rd: Register, r1: Register, r2: Register, },
    sll  { rd: Register, r1: Register, r2: Register, },
    slt  { rd: Register, r1: Register, r2: Register, },
    sltu { rd: Register, r1: Register, r2: Register, },
    xor  { rd: Register, r1: Register, r2: Register, },
    srl  { rd: Register, r1: Register, r2: Register, },
    sra  { rd: Register, r1: Register, r2: Register, },
    or   { rd: Register, r1: Register, r2: Register, },
    and  { rd: Register, r1: Register, r2: Register, },

    // Memory
    lw { rd: Register, mem: RegOffset },
    sw { r1: Register, mem: RegOffset },

    // Branches + some fake branches
    beq  { r1: Register, r2: Register, label: String },
    bne  { r1: Register, r2: Register, label: String },
    blt  { r1: Register, r2: Register, label: String },
    bge  { r1: Register, r2: Register, label: String },
    bltu { r1: Register, r2: Register, label: String },
    bgeu { r1: Register, r2: Register, label: String },
    bgt  { r1: Register, r2: Register, label: String },
    ble  { r1: Register, r2: Register, label: String },
    bgtu { r1: Register, r2: Register, label: String },
    bleu { r1: Register, r2: Register, label: String },

    // 0-branches
    beqz { r1: Register, label: String },
    bnez { r1: Register, label: String },
    bltz { r1: Register, label: String },
    bgez { r1: Register, label: String },
    bgtz { r1: Register, label: String },
    blez { r1: Register, label: String },

    // Unaries
    mv  { rd: Register, r1: Register },
    not { rd: Register, r1: Register },
    neg { rd: Register, r1: Register },

    // Calling and jumping
    jal         { rd: Register, label: String },
    jalr        { rd: Register, addr: RegOffset },
    call        { label: String },
    pseudo_jal  { label: String },
    j           { label: String },
    jr          { rs: Register },
    pseudo_jalr { rs: Register },
    ret         {},
}

macro_rules! instruction_parse_impl {
    // This has to go above the next rule so it matches first
    // Otherwise the next rule will fail to parse ret
    (ret {}) => {
        fn ret(line: &str) -> Result<Instruction, String> {
            if line.trim() == "ret" {
                Ok(Instruction::ret {})
            } else {
                Err("line was not just `ret`".into())
            }
        }
    };

    ( $instr:ident { $( $field:ident: $type:ty ),* $(,)? } ) => {
        fn $instr(line: &str) -> Result<Instruction, String> {
            // We know the ret case is covered so we can call unwrap
            let (op, args) = line.split_once(' ').ok_or("did not find op and args")?;

            // Check opcode
            if op != stringify!($instr) {
                return Err("opcode was wrong".into());
            }

            // We have to take care to parse spaced out instructions
            let mut tokens = args
                .trim()
                // First separate the arguments
                .split(',')
                // Then get rid of spaces
                .map(|tok| tok.replace(" ", ""))
                // Replace *hex integer literals* with decimal integers.
                // Hex integers can occur in register offsets (ex. lw a0, 0x(a0)),
                // but regoffset deals with this in its own parse impl
                .map(
                    |token| parse_int::parse::<i32>(&token).map_or(token, |num| num.to_string())
                );

            Ok(Instruction::$instr {
                $(
                    $field: tokens
                        .next()
                        .ok_or::<String>(concat!("expected ", stringify!($type), ", found empty stream").into())?
                        .parse::<$type>()
                        .map_err(|e| format!("error parsing {}: {}", stringify!($type), e))?
                ),*
            })
        }
    };

    ($( $instr:ident { $( $field:ident: $type:ty ),* $(,)? } ),*) => {
        // Create a parse method for each instruction
        $( instruction_parse_impl!($instr { $( $field: $type ),*  }); )*

        impl FromStr for Instruction {
            type Err = String;

            fn from_str(mut line: &str) -> Result<Instruction, Self::Err> {
                line = line.trim();
                if let Some(index) = line.find('#') {
                    line = &line[..index];
                }
                $(
                    if let Ok(instruction) = $instr(line) {
                        return Ok(instruction);
                    } else {
                        println!("{:?}", $instr(line))
                    }
                )*

                // Deal with instructions that can be parsed in multiple ways
                if line.starts_with("jal") {
                    if let Ok(instruction) = pseudo_jal(&format!("pseudo_{line}")) {
                        return Ok(instruction);
                    }
                }

                if line.starts_with("jalr") {
                    if let Ok(instruction) = pseudo_jalr(&format!("pseudo_{line}")) {
                        return Ok(instruction);
                    }
                }

                Err(format!("No instructions matched"))
            }
        }
    };
}

instruction_parse_impl! {
    // Register immediate
    addi  { rd: Register, r1: Register, imm: i32, },
    slti  { rd: Register, r1: Register, imm: i32, },
    sltiu { rd: Register, r1: Register, imm: i32, },
    xori  { rd: Register, r1: Register, imm: i32, },
    ori   { rd: Register, r1: Register, imm: i32, },
    andi  { rd: Register, r1: Register, imm: i32, },
    slli  { rd: Register, r1: Register, imm: i32, },
    srli  { rd: Register, r1: Register, imm: i32, },
    srai  { rd: Register, r1: Register, imm: i32, },

    // Loady bois
    lui { rd: Register, imm: i32, },
    li  { rd: Register, imm: i32, },

    // Register register
    add  { rd: Register, r1: Register, r2: Register, },
    sub  { rd: Register, r1: Register, r2: Register, },
    sll  { rd: Register, r1: Register, r2: Register, },
    slt  { rd: Register, r1: Register, r2: Register, },
    sltu { rd: Register, r1: Register, r2: Register, },
    xor  { rd: Register, r1: Register, r2: Register, },
    srl  { rd: Register, r1: Register, r2: Register, },
    sra  { rd: Register, r1: Register, r2: Register, },
    or   { rd: Register, r1: Register, r2: Register, },
    and  { rd: Register, r1: Register, r2: Register, },

    // Memory
    lw { rd: Register, mem: RegOffset },
    sw { r1: Register, mem: RegOffset },

    // Branches + some fake branches
    beq  { r1: Register, r2: Register, label: String },
    bne  { r1: Register, r2: Register, label: String },
    blt  { r1: Register, r2: Register, label: String },
    bge  { r1: Register, r2: Register, label: String },
    bltu { r1: Register, r2: Register, label: String },
    bgeu { r1: Register, r2: Register, label: String },
    bgt  { r1: Register, r2: Register, label: String },
    ble  { r1: Register, r2: Register, label: String },
    bgtu { r1: Register, r2: Register, label: String },
    bleu { r1: Register, r2: Register, label: String },

    // 0-branches
    beqz { r1: Register, label: String },
    bnez { r1: Register, label: String },
    bltz { r1: Register, label: String },
    bgez { r1: Register, label: String },
    bgtz { r1: Register, label: String },
    blez { r1: Register, label: String },

    // Unaries
    mv  { rd: Register, r1: Register },
    not { rd: Register, r1: Register },
    neg { rd: Register, r1: Register },

    // Calling and jumping
    jal         { rd: Register, label: String },
    jalr        { rd: Register, addr: RegOffset },
    call        { label: String },
    pseudo_jal  { label: String },
    j           { label: String },
    jr          { rs: Register },
    pseudo_jalr { rs: Register },
    ret         {}
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub enum Riscv {
    Instruction(Instruction),
    Label(String),
    Word { value: i32, location: i32 },
    Comment(String),
}

impl FromStr for Riscv {
    type Err = String;

    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
        s = s.trim();
        if let Some(comment) = s.strip_prefix('#') {
            Ok(Riscv::Comment(comment.into()))
        } else if let Some(label) = s.strip_suffix(':') {
            if label.chars().all(|c| c.is_alphanumeric() || c == '_') {
                Ok(Riscv::Label(label.into()))
            } else {
                Err("non alphanumeric character detected in label".into())
            }
        } else if let Some(_directive) = s.strip_prefix('.') {
            todo!()
        } else {
            s.parse::<Instruction>().map(Riscv::Instruction)
        }
    }
}

pub fn parse_program(program: String) -> Vec<Result<Riscv, String>> {
    program
        .split('\n')
        // Strop comments
        .map(|s| {
            if let Some(index) = s.find('#') {
                &s[..index]
            } else {
                s
            }
        })
        .map(|s| {
            if let Some(index) = s.find("//") {
                &s[..index]
            } else {
                s
            }
        })
        .filter(|s| !s.trim().is_empty())
        .map(|line| {
            println!("{line}");
            line.trim()
                .parse()
                .map_err(|err| format!("error parsing {line}: {err}"))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn beq() {
        assert_eq!(
            Instruction::beq {
                r1: Register::a0,
                r2: Register::a0,
                label: "hello".into()
            },
            super::beq("beq a0, a0, hello").unwrap()
        )
    }

    #[test]
    fn addi() {
        assert_eq!(
            Instruction::addi {
                rd: Register::a0,
                r1: Register::a0,
                imm: 0x420
            },
            super::addi("addi a0, a0, 0x420").unwrap()
        );
    }

    #[test]
    fn ret() {
        assert_eq!(Instruction::ret {}, super::ret("ret").unwrap())
    }

    #[test]
    fn lw() {
        assert_eq!(
            Instruction::lw {
                rd: Register::a0,
                mem: RegOffset {
                    reg: Register::a1,
                    offset: 2
                }
            },
            super::lw("lw a0, 2(a1)").unwrap()
        )
    }

    #[test]
    fn lw_negative() {
        assert_eq!(
            Instruction::lw {
                rd: Register::a0,
                mem: RegOffset {
                    reg: Register::a1,
                    offset: -2
                }
            },
            super::lw("lw a0, -2(a1)").unwrap()
        );
        assert_eq!(
            Instruction::lw {
                rd: Register::a0,
                mem: RegOffset {
                    reg: Register::a1,
                    offset: -33
                }
            },
            super::lw("lw a0, 0x-21(a1)").unwrap()
        );
    }

    #[test]
    fn addi2() {
        assert!(parse_program(
            "addi x0, x0, 1
        addi x1, x2, 5"
                .into()
        )
        .iter()
        .all(Result::is_ok));
        assert!("slli x1, x2, 5".to_owned().parse::<Riscv>().is_ok())
    }

    #[test]
    fn lw_whitespace() {
        assert_eq!(
            Instruction::lw {
                rd: Register::a0,
                mem: RegOffset {
                    reg: Register::a1,
                    offset: 2
                }
            },
            super::lw("lw    a0,    2(   a1    )").unwrap()
        )
    }

    #[test]
    fn parse() {
        let assembly = vec![
            "li a2, 1",
            "addi a1, a1, -1",
            "li a2, 0 # swapped = 0",
            "li a3, 0 # i = 0",
            "bge a3, a1, end_for # end the for loop if i >= n",
            "slli a4, a3, 2 # offset = i << 2 = i * 4",
            "add a4, a4, a0 # a4 now stores ptr + offset",
            "lw a5, 0(a4) # p[i]",
            "lw a6, 4(a4) # p[i+1]",
            "bge a6, a5, noswap # go to noswap",
            "mv a7, a5 # tmp = p[i]",
            "sw a6, 0(a4) # p[i] = p[i+1]",
            "sw a7, 4(a4) # p[i+1] = p[i]",
            "li a2, 1 # swapped = 1",
            "addi sp, sp, -36",
            "sw a0, 0(sp)",
            "sw a1, 4(sp)",
            "sw a2, 8(sp)",
            "sw a3, 12(sp)",
            "sw a4, 16(sp)",
            "sw a5, 20(sp)",
            "sw a6, 24(sp)",
            "sw a7, 28(sp)",
            "sw ra, 32(sp)",
            "jal arrayViz",
        ];
        for line in assembly {
            println!("{line}");
            str::parse::<Instruction>(line).unwrap();
        }
    }

    #[test]
    fn big() {
        let parsed = parse_program(include_str!("test.s").to_owned());
        for res in parsed.iter() {
            assert!(res.is_ok())
        }
    }
}
