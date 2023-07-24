use anyhow::bail;
use std::str::FromStr;
use thiserror::Error;

use crate::lex::{Lexer, TokenInner, LexerIter};

use serde::{Deserialize, Serialize};

use crate::lex::Token;

fn is_reg_imm(instr: &str) -> bool {
    [
        "addi", "slti", "sltiu", "xori", "ori", "andi", "slli", "srli", "srai",
    ]
    .contains(&instr)
}

fn is_reg_reg(instr: &str) -> bool {
    [
        "add", "sub", "sll", "slt", "sltu", "xor", "srl", "sra", "or", "and",
    ]
    .contains(&instr)
}

fn is_branch(instr: &str) -> bool {
    [
        "beq", "bne", "blt", "bge", "bltu", "bgeu", "bgt", "ble", "bgtu", "bleu",
    ]
    .contains(&instr)
}

fn is_branch_zero(instr: &str) -> bool {
    ["beqz", "bnez", "bltz", "bgez", "bgtz", "blez"].contains(&instr)
}

fn is_unary(instr: &str) -> bool {
    ["mv", "neg", "not"].contains(&instr)
}

fn is_mem_op(instr: &str) -> bool {
    ["lw", "lh", "lb", "sw", "sh", "sb"].contains(&instr)
}

fn is_load(instr: &str) -> bool {
    ["lui", "li"].contains(&instr)
}

#[allow(non_camel_case_types)]
#[rustfmt::skip]
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum Register {
    x0, ra, sp, gp, tp,
    t0, t1, t2, t3, t4, t5, t6,
    a0, a1, a2, a3, a4, a5, a6, a7,
    s0, s1, s2, s3, s4, s5, s6, s7, s8, s9, s10, s11,
}

#[derive(Error, Debug)]
pub enum RegisterParseError {
    // Note: we can't include the actual offending token here because anyhow
    // requires it's error's be 'static but the token would only live for 'a
    #[error("can only parse register out of ident, but got '{0}'")]
    InvalidToken(String),
    #[error("'{0}' is not a valid register")]
    ParseError(String),
}

impl TryFrom<Token<'_>> for Register {
    type Error = RegisterParseError;

    fn try_from(token: Token) -> Result<Self, Self::Error> {
        match token.inner() {
            TokenInner::Ident(ident) => ident.parse().map_err(RegisterParseError::ParseError),
            other => Err(RegisterParseError::InvalidToken(other.to_string())),
        }
    }
}

/// Parse a register from a string
macro_rules! register_parse_impl {
    ($( ($reg:ident = $xreg:ident) )*) => {
        impl FromStr for Register {
            type Err = String;
            fn from_str(s: &str) -> Result<Register, Self::Err> {
                match s.trim() {
                    $(
                        stringify!($reg) | stringify!($xreg)
                            => Ok(Register::$reg),
                    )*
                    "zero" => Ok(Register::x0),
                    unknown => Err(format!("unrecognized register {unknown}"))
                }
            }
        }
    }
}

register_parse_impl! {
   (x0 = x0) (ra = x1) (sp = x2) (gp = x3) (tp = x4)

   (t0 = x5) (t1 = x6) (t2 = x7) (t3 = x28) (t4 = x29) (t5 = x30) (t6 = x31)

   (a0 = x10) (a1 = x11) (a2 = x12) (a3 = x13)
   (a4 = x14) (a5 = x15) (a6 = x16) (a7 = x17)

   (s0 = x8) (s1 = x9) (s2 = x18) (s3 = x19) (s4 = x20) (s5 = x21) (s6 = x22)
   (s7 = x23) (s8 = x24) (s9 = x25) (s10 = x26) (s11 = x27)
}

#[rustfmt::skip]
#[allow(non_camel_case_types)]
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub enum Instruction<'a> {
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
    lw { rd: Register, offset: i32, r1: Register  },
    lh { rd: Register, offset: i32, r1: Register  },
    lb { rd: Register, offset: i32, r1: Register  },
    sw { r2: Register, offset: i32, r1: Register },
    sh { r2: Register, offset: i32, r1: Register },
    sb { r2: Register, offset: i32, r1: Register },

    // Branches + some fake branches
    beq  { r1: Register, r2: Register, label: &'a str },
    bne  { r1: Register, r2: Register, label: &'a str },
    blt  { r1: Register, r2: Register, label: &'a str },
    bge  { r1: Register, r2: Register, label: &'a str },
    bltu { r1: Register, r2: Register, label: &'a str },
    bgeu { r1: Register, r2: Register, label: &'a str },
    bgt  { r1: Register, r2: Register, label: &'a str },
    ble  { r1: Register, r2: Register, label: &'a str },
    bgtu { r1: Register, r2: Register, label: &'a str },
    bleu { r1: Register, r2: Register, label: &'a str },

    // Loady bois
    lui { rd: Register, imm: i32, },
    li  { rd: Register, imm: i32, },

    // 0-branches
    beqz { r1: Register, label: &'a str },
    bnez { r1: Register, label: &'a str },
    bltz { r1: Register, label: &'a str },
    bgez { r1: Register, label: &'a str },
    bgtz { r1: Register, label: &'a str },
    blez { r1: Register, label: &'a str },

    // Unaries
    mv  { rd: Register, r1: Register },
    not { rd: Register, r1: Register },
    neg { rd: Register, r1: Register },

    // Calling and jumping
    call        { label: &'a str },
    // Note: if a register is not provided, assume rd
    jal         { rd: Register, label: &'a str },
    // Note: if a register is not provided, assume 0(rd)
    jalr        { rd: Register, offset: i32, r1: Register },
    j           { label: &'a str },
    jr          { rs: Register },
    ret         {},
}

/// An item of RISC-V assembly, either an instruction or label (for now)
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub enum Item<'a> {
    #[serde(borrow)]
    Instruction(Instruction<'a>),
    Label(&'a str),
}

pub fn parse_item<'a>(tokens: &mut LexerIter<'a>) -> anyhow::Result<Item<'a>> {
    //let mut tokens = Lexer::new(source).into_iter();
    while matches!(
        tokens.peek(),
        Some(Ok(Token {
            inner: TokenInner::Comment(_),
            ..
        }))
    ) {
        tokens.next().unwrap()?;
    }
    let ident = tokens.ident()?.unwrap_ident();
    if let Ok(TokenInner::Colon) = tokens.colon().map(|token| token.inner()) {
        return Ok(Item::Label(ident));
    }
    Ok(Item::Instruction(if is_reg_imm(ident) {
        let rd = tokens.ident()?.try_into()?;
        let _ = tokens.comma()?;
        let r1 = tokens.ident()?.try_into()?;
        let _ = tokens.comma()?;
        let negative = tokens.minus().is_ok();
        let mut imm: i32 = tokens.constant()?.unwrap_constant();
        if negative {
            imm = -imm
        }
        match ident {
            "slti" => Instruction::slti { rd, r1, imm },
            "addi" => Instruction::addi { rd, r1, imm },
            "sltiu" => Instruction::sltiu { rd, r1, imm },
            "xori" => Instruction::xori { rd, r1, imm },
            "ori" => Instruction::ori { rd, r1, imm },
            "andi" => Instruction::andi { rd, r1, imm },
            "slli" => Instruction::slli { rd, r1, imm },
            "srli" => Instruction::srli { rd, r1, imm },
            "srai" => Instruction::srai { rd, r1, imm },
            other => panic!("unkown register-immediate instruction: {other}"),
        }
    } else if is_reg_reg(ident) {
        let rd = tokens.ident()?.try_into()?;
        let _ = tokens.comma()?;
        let r1 = tokens.ident()?.try_into()?;
        let _ = tokens.comma()?;
        let r2 = tokens.ident()?.try_into()?;
        match ident {
            "add" => Instruction::and { rd, r1, r2 },
            "sub" => Instruction::sub { rd, r1, r2 },
            "sll" => Instruction::sll { rd, r1, r2 },
            "slt" => Instruction::slt { rd, r1, r2 },
            "sltu" => Instruction::sltu { rd, r1, r2 },
            "xor" => Instruction::xor { rd, r1, r2 },
            "srl" => Instruction::srl { rd, r1, r2 },
            "sra" => Instruction::sra { rd, r1, r2 },
            "or" => Instruction::or { rd, r1, r2 },
            "and" => Instruction::and { rd, r1, r2 },
            other => panic!("unknown register-register instruction: {other}"),
        }
    } else if is_branch(ident) {
        let r1 = tokens.ident()?.try_into()?;
        let _ = tokens.comma()?;
        let r2 = tokens.ident()?.try_into()?;
        let _ = tokens.comma()?;
        let label = tokens.ident()?.unwrap_ident();
        match ident {
            "beq" => Instruction::beq { r1, r2, label },
            "bne" => Instruction::beq { r1, r2, label },
            "blt" => Instruction::beq { r1, r2, label },
            "bge" => Instruction::beq { r1, r2, label },
            "bltu" => Instruction::beq { r1, r2, label },
            "bgeu" => Instruction::beq { r1, r2, label },
            "bgt" => Instruction::beq { r1, r2, label },
            "ble" => Instruction::beq { r1, r2, label },
            "bgtu" => Instruction::beq { r1, r2, label },
            "bleu" => Instruction::beq { r1, r2, label },
            other => panic!("unknown branch instruction: {other}"),
        }
    } else if is_branch_zero(ident) {
        let r1 = tokens.ident()?.try_into()?;
        let _ = tokens.comma()?;
        let label = tokens.ident()?.unwrap_ident();
        match ident {
            "beqz" => Instruction::beqz { r1, label },
            "bnez" => Instruction::beqz { r1, label },
            "bltz" => Instruction::beqz { r1, label },
            "bgez" => Instruction::beqz { r1, label },
            "bgtz" => Instruction::beqz { r1, label },
            "blez" => Instruction::beqz { r1, label },
            other => panic!("unkown branch-zero instruction: {other}"),
        }
    } else if is_unary(ident) {
        let rd = tokens.ident()?.try_into()?;
        let _ = tokens.comma()?;
        let r1 = tokens.ident()?.try_into()?;
        match ident {
            "mv" => Instruction::mv { rd, r1 },
            "not" => Instruction::mv { rd, r1 },
            "neg" => Instruction::mv { rd, r1 },
            other => panic!("unkown unary instruction: {other}"),
        }
    } else if is_mem_op(ident) {
        let reg = tokens.ident()?.try_into()?;
        let _ = tokens.comma()?;
        let offset = tokens.constant()?.unwrap_constant();
        let _ = tokens.left_paren()?;
        let r1 = tokens.ident()?.try_into()?;
        let _ = tokens.right_paren()?;
        match ident {
            "sw" => Instruction::sw {
                r2: reg,
                offset,
                r1,
            },
            "sh" => Instruction::sh {
                r2: reg,
                offset,
                r1,
            },
            "sb" => Instruction::sb {
                r2: reg,
                offset,
                r1,
            },
            "lw" => Instruction::lw {
                rd: reg,
                offset,
                r1,
            },
            "lh" => Instruction::lh {
                rd: reg,
                offset,
                r1,
            },
            "lb" => Instruction::lb {
                rd: reg,
                offset,
                r1,
            },
            other => panic!("unknown memory instruction {other}"),
        }
    } else if is_load(ident) {
        let rd = tokens.ident()?.try_into()?;
        let _ = tokens.comma()?;
        let imm = tokens.constant()?.unwrap_constant();
        match ident {
            "lui" => Instruction::lui { rd, imm },
            "li" => Instruction::li { rd, imm },
            other => panic!("unknown memory instruction {other}"),
        }
    } else {
        // call        { label: &'a str },
        // jal         { rd: Register, label: &'a str },
        // jalr        { rd: Register, offset: i32, r1: Register },
        // j           { label: &'a str },
        // jr          { rs: Register },
        // ret         {},
        match ident {
            "call" => {
                let label = tokens.ident()?.unwrap_ident();
                Instruction::call { label }
            }
            // Note: if a register is not provided, assume rd
            "jal" => {
                let ident = tokens.ident()?;
                if let Ok(rd) = ident.clone().try_into() {
                    // Register was provided, continue
                    let _ = tokens.comma()?;
                    let label = tokens.ident()?.unwrap_ident();
                    Instruction::jal { rd, label }
                } else {
                    // Assume register is rd
                    Instruction::jal {
                        rd: "ra".parse().expect(""),
                        label: ident.unwrap_ident(),
                    }
                }
            }
            // Note: if a register is not provided, assume 0(rd)
            "jalr" => {
                let reg = tokens.ident()?.try_into()?;
                if let Ok(TokenInner::Comma) = tokens.comma().map(|token| token.inner()) {
                    let offset = tokens.constant()?.unwrap_constant();
                    let _ = tokens.left_paren()?;
                    let r1 = tokens.ident()?.try_into()?;
                    let _ = tokens.right_paren()?;
                    Instruction::jalr {
                        rd: reg,
                        offset,
                        r1,
                    }
                } else {
                    Instruction::jalr {
                        rd: "ra".parse().expect(""),
                        offset: 0,
                        r1: reg,
                    }
                }
            }
            "j" => {
                let label = tokens.ident()?.unwrap_ident();
                Instruction::j { label }
            }
            "jr" => {
                let rs = tokens.ident()?.try_into()?;
                Instruction::jr { rs }
            }
            "ret" => Instruction::ret {},
            other => bail!("unknown instruction: {other}"),
        }
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
}
