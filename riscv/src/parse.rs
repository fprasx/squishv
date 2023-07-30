use anyhow::{anyhow, bail, Context};
use core::fmt;
use std::collections::HashMap;
use thiserror::Error;

use crate::lex::{Lexer, Span, TokenInner};

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

/// Implement parse, display for Register
macro_rules! register_impls {
    ($( ($reg:ident = $xreg:ident) )*) => {
        impl std::str::FromStr for $crate::parse::Register {
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
        impl std::fmt::Display for $crate::parse::Register {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self {
                    $(
                        Register::$reg => write!(f, "{}", stringify!($reg)),
                    )*
                }
            }
        }
    }
}

register_impls! {
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

impl<'a> fmt::Display for Instruction<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instruction::addi { rd, r1, imm } => write!(f, "addi {rd}, {r1}, {imm}"),
            Instruction::slti { rd, r1, imm } => write!(f, "slti {rd}, {r1}, {imm}"),
            Instruction::sltiu { rd, r1, imm } => write!(f, "sltiu {rd}, {r1}, {imm}"),
            Instruction::xori { rd, r1, imm } => write!(f, "xori {rd}, {r1}, {imm}"),
            Instruction::ori { rd, r1, imm } => write!(f, "ori {rd}, {r1}, {imm}"),
            Instruction::andi { rd, r1, imm } => write!(f, "andi {rd}, {r1}, {imm}"),
            Instruction::slli { rd, r1, imm } => write!(f, "slli {rd}, {r1}, {imm}"),
            Instruction::srli { rd, r1, imm } => write!(f, "srli {rd}, {r1}, {imm}"),
            Instruction::srai { rd, r1, imm } => write!(f, "srai {rd}, {r1}, {imm}"),
            Instruction::add { rd, r1, r2 } => write!(f, "add {rd}, {r1}, {r2}"),
            Instruction::sub { rd, r1, r2 } => write!(f, "sub {rd}, {r1}, {r2}"),
            Instruction::sll { rd, r1, r2 } => write!(f, "sll {rd}, {r1}, {r2}"),
            Instruction::slt { rd, r1, r2 } => write!(f, "slt {rd}, {r1}, {r2}"),
            Instruction::sltu { rd, r1, r2 } => write!(f, "sltu {rd}, {r1}, {r2}"),
            Instruction::xor { rd, r1, r2 } => write!(f, "xor {rd}, {r1}, {r2}"),
            Instruction::srl { rd, r1, r2 } => write!(f, "swl {rd}, {r1}, {r2}"),
            Instruction::sra { rd, r1, r2 } => write!(f, "sra {rd}, {r1}, {r2}"),
            Instruction::or { rd, r1, r2 } => write!(f, "or {rd}, {r1}, {r2}"),
            Instruction::and { rd, r1, r2 } => write!(f, "and {rd}, {r1}, {r2}"),
            Instruction::lw { rd, offset, r1 } => write!(f, "lw {rd}, {offset}({r1})"),
            Instruction::lh { rd, offset, r1 } => write!(f, "lh {rd}, {offset}({r1})"),
            Instruction::lb { rd, offset, r1 } => write!(f, "lb {rd}, {offset}({r1})"),
            Instruction::sw { r2, offset, r1 } => write!(f, "sw {r2}, {offset}({r1})"),
            Instruction::sh { r2, offset, r1 } => write!(f, "sh {r2}, {offset}({r1})"),
            Instruction::sb { r2, offset, r1 } => write!(f, "sb {r2}, {offset}({r1})"),
            Instruction::beq { r1, r2, label } => write!(f, "beq {r1}, {r2}, {label}"),
            Instruction::bne { r1, r2, label } => write!(f, "bne {r1}, {r2}, {label}"),
            Instruction::blt { r1, r2, label } => write!(f, "blt {r1}, {r2}, {label}"),
            Instruction::bge { r1, r2, label } => write!(f, "bge {r1}, {r2}, {label}"),
            Instruction::bltu { r1, r2, label } => write!(f, "bltu {r1}, {r2}, {label}"),
            Instruction::bgeu { r1, r2, label } => write!(f, "bgeuw {r1}, {r2}, {label}"),
            Instruction::bgt { r1, r2, label } => write!(f, "bgt {r1}, {r2}, {label}"),
            Instruction::ble { r1, r2, label } => write!(f, "ble {r1}, {r2}, {label}"),
            Instruction::bgtu { r1, r2, label } => write!(f, "bgtu {r1}, {r2}, {label}"),
            Instruction::bleu { r1, r2, label } => write!(f, "bleu {r1}, {r2}, {label}"),
            Instruction::lui { rd, imm } => write!(f, "lui {rd} {imm}"),
            Instruction::li { rd, imm } => write!(f, "li {rd} {imm}"),
            Instruction::beqz { r1, label } => write!(f, "beqz {r1}, {label}"),
            Instruction::bnez { r1, label } => write!(f, "bnez {r1}, {label}"),
            Instruction::bltz { r1, label } => write!(f, "bltz {r1}, {label}"),
            Instruction::bgez { r1, label } => write!(f, "bgez {r1}, {label}"),
            Instruction::bgtz { r1, label } => write!(f, "bgtz {r1}, {label}"),
            Instruction::blez { r1, label } => write!(f, "blez {r1}, {label}"),
            Instruction::mv { rd, r1 } => write!(f, "mv {rd}, {r1}"),
            Instruction::not { rd, r1 } => write!(f, "mv {rd}, {r1}"),
            Instruction::neg { rd, r1 } => write!(f, "mv {rd}, {r1}"),
            Instruction::call { label } => write!(f, "call {label}"),
            Instruction::jal { rd, label } => write!(f, "jal {rd}, {label}"),
            Instruction::jalr { rd, offset, r1 } => write!(f, "jalr {rd}, {offset}({r1})"),
            Instruction::j { label } => write!(f, "j {label}"),
            Instruction::jr { rs } => write!(f, "jr {rs}"),
            Instruction::ret {} => write!(f, "ret"),
        }
    }
}

/// An item of RISC-V assembly, either an instruction or label (for now)
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub enum Item<'a> {
    #[serde(borrow)]
    Instruction(Instruction<'a>),

    // Include span info so that we can emit better error messages during the
    // post-processing stage of parsing, where we check that all labels accessed
    // actually exist and that no labels are defined more than once.
    Label {
        name: &'a str,
        span: Span,
    },
}

impl<'a> Item<'a> {
    /// Access the inner instruction. Panic if not called on an instruction.
    pub fn get_instruction(&self) -> &Instruction<'a> {
        match self {
            Item::Instruction(i) => i,
            Item::Label { .. } => unreachable!("unwrap_instruction called on label"),
        }
    }

    /// Access the inner label. Panic if not called on an label.
    pub fn get_label(self) -> &'a str {
        match self {
            Item::Instruction(_) => unreachable!("unwrap_label called on instruction"),
            Item::Label { name, .. } => name,
        }
    }
}

type ParseResult<'a> = anyhow::Result<Item<'a>>;

pub fn parse_item<'a>(tokens: &mut Lexer<'a>) -> ParseResult<'a> {
    // Skip comments
    while matches!(
        tokens.peek(),
        Some(Ok(Token {
            inner: TokenInner::HashComment(_) | TokenInner::SlashComment(_),
            ..
        }))
    ) {
        tokens.next().unwrap()?;
    }

    // Parsing a label
    let ident = tokens.ident()?;
    let (ident, span) = ident.unwrap_ident();
    if let Ok(TokenInner::Colon) = tokens.colon().map(|token| token.inner()) {
        return Ok(Item::Label { name: ident, span });
    }

    // Parsing anything else

    let instruction = if is_reg_imm(ident) {
        let rd = tokens.ident()?.try_into()?;
        let _ = tokens.comma()?;
        let r1 = tokens.ident()?.try_into()?;
        let _ = tokens.comma()?;
        let neg = tokens.minus().is_ok();
        let mut imm: i32 = tokens.constant()?.unwrap_constant().0;
        if neg {
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
        let label = tokens.ident()?.unwrap_ident().0;
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
        let label = tokens.ident()?.unwrap_ident().0;
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
        let neg = tokens.minus().is_ok();
        let mut offset = tokens.constant()?.unwrap_constant().0;
        if neg {
            offset = -offset
        }
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
        let neg = tokens.minus().is_ok();
        let mut imm = tokens.constant()?.unwrap_constant().0;
        if neg {
            imm = -imm
        }
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
                let label = tokens.ident()?.unwrap_ident().0;
                Instruction::call { label }
            }
            // Note: if a register is not provided, assume rd
            "jal" => {
                let ident = tokens.ident()?;
                if let Ok(rd) = ident.clone().try_into() {
                    // Register was provided, continue
                    let _ = tokens.comma()?;
                    let label = tokens.ident()?.unwrap_ident().0;
                    Instruction::jal { rd, label }
                } else {
                    // Assume register is ra
                    Instruction::jal {
                        rd: "ra".parse().expect(""),
                        label: ident.unwrap_ident().0,
                    }
                }
            }
            // Note: if a register is not provided, assume 0(rd)
            "jalr" => {
                let reg = tokens.ident()?.try_into()?;
                if let Ok(TokenInner::Comma) = tokens.comma().map(|token| token.inner()) {
                    let neg = tokens.minus().is_ok();
                    let mut offset = tokens.constant()?.unwrap_constant().0;
                    if neg {
                        offset = -offset
                    }
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
                let label = tokens.ident()?.unwrap_ident().0;
                Instruction::j { label }
            }
            "jr" => {
                let rs = tokens.ident()?.try_into()?;
                Instruction::jr { rs }
            }
            "ret" => Instruction::ret {},
            other => bail!("unknown instruction: {other}"),
        }
    };
    Ok(Item::Instruction(instruction))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program<'a> {
    // The values of this map are indices into `items`. They point to the item
    // in items corresponding to the label with the keyed name.
    labels: HashMap<&'a str, usize>,
    items: Vec<Item<'a>>,
}

impl<'a> Program<'a> {
    // Get the raw items out of the lexer
    fn parse_items(source: &'a str) -> anyhow::Result<Vec<Item>> {
        let mut items = vec![];
        let mut lexer = Lexer::new(source);
        while !lexer.finished() {
            items.push(parse_item(&mut lexer)?);
        }
        Ok(items)
    }

    pub fn parse(source: &'a str) -> anyhow::Result<Program> {
        // We use this to check if any labels are defined multiple times
        let mut labels2pcs: HashMap<&str, usize> = HashMap::new();
        let mut labels2spans: HashMap<&'a str, Vec<Span>> = HashMap::new();
        let items = Program::parse_items(source).context("failed to parse item")?;
        for (i, item) in items.iter().enumerate() {
            if let Item::Label { name, span } = &item {
                labels2pcs.insert(name, i);

                // Record the spans where each label is defined; there should only be one
                // for each label
                labels2spans
                    .entry(name)
                    .and_modify(|spans| spans.push(span.clone()))
                    .or_insert(vec![span.clone()]);
            }
        }

        let mut errors: Vec<String> = vec![];
        for (name, spans) in labels2spans.iter() {
            if spans.len() != 1 {
                let mut error = format!("label <{name}> defined multiple times at:\n");
                error.push_str(
                    &spans
                        .iter()
                        .map(|span| format!("\t{span}"))
                        .collect::<Vec<String>>()
                        .join("\n"),
                );
                errors.push(error);
            }
        }

        for (pc, instr) in items.iter().enumerate() {
            let Item::Instruction(instr) = instr else {
                continue;
            };
            let label = match instr {
                Instruction::beq { label, .. } => label,
                Instruction::bne { label, .. } => label,
                Instruction::blt { label, .. } => label,
                Instruction::bge { label, .. } => label,
                Instruction::bltu { label, .. } => label,
                Instruction::bgeu { label, .. } => label,
                Instruction::bgt { label, .. } => label,
                Instruction::ble { label, .. } => label,
                Instruction::bgtu { label, .. } => label,
                Instruction::bleu { label, .. } => label,
                Instruction::beqz { label, .. } => label,
                Instruction::bnez { label, .. } => label,
                Instruction::bltz { label, .. } => label,
                Instruction::bgez { label, .. } => label,
                Instruction::bgtz { label, .. } => label,
                Instruction::blez { label, .. } => label,
                Instruction::call { label } => label,
                Instruction::jal { label, .. } => label,
                Instruction::j { label } => label,
                _ => continue,
            };
            if !labels2spans.contains_key(label) {
                errors.push(format!(
                    // pad with 10 zeroes because the 0x prefix takes up 2 chars
                    "undefined label <{label}> at pc {:#010x}: {}",
                    pc * 4,
                    instr
                ))
            }
        }

        if !errors.is_empty() {
            return Err(anyhow!(errors.join("\n")).context("failed to parse"));
        }

        Ok(Program {
            items,
            labels: labels2pcs,
        })
    }

    pub fn at<'prog>(&'prog self, label: &str) -> Option<&'prog Instruction> {
        return self
            .labels
            .get(label)
            .and_then(|pc| self.items.get(*pc))
            .map(Item::get_instruction);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    // vec! like syntax for a hashmap
    macro_rules! map {
        ($($key:expr => $val:expr),* $(,)?) => {
            HashMap::from([$((($key, $val)),)*])
        };
    }

    #[test]
    fn empty() {
        assert_eq!(
            Program::parse("").unwrap(),
            Program {
                items: vec![],
                labels: map![]
            }
        )
    }

    #[test]
    fn labels() {
        assert_eq!(
            Program::parse(indoc! {"
                checka:
                loopa:
                checkb:
                loopb:
            "})
            .unwrap(),
            Program {
                items: vec![
                    Item::Label {
                        name: "checka",
                        span: Span::new(1, 1..7)
                    },
                    Item::Label {
                        name: "loopa",
                        span: Span::new(2, 1..6)
                    },
                    Item::Label {
                        name: "checkb",
                        span: Span::new(3, 1..7)
                    },
                    Item::Label {
                        name: "loopb",
                        span: Span::new(4, 1..6)
                    },
                ],
                labels: map![
                    "checka" => 0,
                    "loopa"=> 1,
                    "checkb"=> 2,
                    "loopb"=> 3,
                ]
            }
        );
    }

    #[test]
    fn repeate_label() {
        assert!(Program::parse(indoc! {"
                repeated:
                repeated:
                beqz zero, repeated
        "})
        .is_err());
    }

    #[test]
    fn missing_label() {
        assert!(Program::parse(indoc! {"
            bne t1, x4, missing
        "})
        .is_err());
    }

    #[test]
    fn instructions() {
        let program = Program::parse(indoc! {""}).unwrap();
    }

    #[test]
    fn fuzz() {
        assert!(Program::parse(include_str!("../tests/test.s")).is_ok());
        assert!(Program::parse(include_str!("../tests/random.s")).is_ok());
    }

    #[test]
    fn mixed() {
        assert_eq!(
            Program::parse(indoc! {"
                addi zero, sp, 1
                addi zero, sp, 2
                label:
                beqz a0, label
            "})
            .unwrap(),
            Program {
                items: vec![
                    Item::Instruction(Instruction::addi {
                        rd: Register::x0,
                        r1: Register::sp,
                        imm: 1
                    }),
                    Item::Instruction(Instruction::addi {
                        rd: Register::x0,
                        r1: Register::sp,
                        imm: 2
                    }),
                    Item::Label {
                        name: "label",
                        span: Span::new(3, 1..6)
                    },
                    Item::Instruction(Instruction::beqz {
                        r1: Register::a0,
                        label: "label"
                    }),
                ],
                labels: map![
                    "label"=> 2,
                ]
            }
        );
    }
}
