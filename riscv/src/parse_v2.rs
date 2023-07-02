use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};

const REG_IMM: [&str; 9] = [
    "addi", "slti", "sltiu", "xori", "ori", "andi", "slli", "srli", "srai",
];

const REG_REG: [&str; 10] = [
    "add", "sub", "sll", "slt", "sltu", "xor", "srl", "sra", "or", "and",
];

const BRANCH: [&str; 10] = [
    "beq", "bne", "blt", "bge", "bltu", "bgeu", "bgt", "ble", "bgtu", "bleu",
];
const BRANCH_ZERO: [&str; 6] = ["beqz", "bnez", "bltz", "bgez", "bgtz", "blez"];

#[allow(non_camel_case_types)]
#[rustfmt::skip]
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum Register {
    x0, ra, sp, gp, tp,
    t0, t1, t2, t3, t4, t5, t6,
    a0, a1, a2, a3, a4, a5, a6, a7,
    s0, s1, s2, s3, s4, s5, s6, s7, s8, s9, s10, s11,
}

#[allow(non_camel_case_types)]

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

// All the empty parenthesese might look a little funky but they are there to
// make macro implementations nicer.
#[derive(Debug, Clone, Copy)]
pub enum Token<'a> {
    RightParen(),
    LeftParen(),
    Comma(),
    Colon(),
    Minus(),
    Constant(i32),
    Ident(&'a str),
}

// Functions for attempting to get a certain token from the lexer. If the next
// token is the desired token, the stream will be advanced and the token will be
// returned. Otherwise the stream is not advanced.
macro_rules! parse_token {
    ($($tokentype:ident => $tokenfn:ident)+) => {
        $(
            impl<'a> Lexer<'a> {
                pub fn $tokenfn(&mut self) -> Result<Token<'a>, String> {
                    match self.peek() {
                        // Unwrap is safe as we already peeked, we just use next
                        // to advance the stream
                        Some(Token::$tokentype(..)) => Ok(self.next().unwrap()),
                        Some(other) => Err(format!("Expected {}, found {other}", stringify!($tokentype))),
                        None => Err(format!("Expected {}, but ran out of input", stringify!($tokentype))),
                    }
                }
            }
        )*
    };
}

// Keep this in sync with the definition of Token
parse_token! {
    Comma => comma
    Colon => colon
    RightParen => right_paren
    LeftParen  => left_paren
    Ident => ident
    Constant => constant
    Minus => minus
}

impl<'a> Token<'a> {
    pub fn unwrap_ident(self) -> &'a str {
        match self {
            Token::Ident(inner) => inner,
            other => panic!("called unwrap ident on a {other}"),
        }
    }

    pub fn unwrap_constant(self) -> i32 {
        match self {
            Token::Constant(inner) => inner,
            other => panic!("called unwrap ident on a {other}"),
        }
    }
}

impl Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::RightParen() => write!(f, "("),
            Token::LeftParen() => write!(f, ")"),
            Token::Comma() => write!(f, ","),
            Token::Colon() => write!(f, ":"),
            Token::Minus() => write!(f, "-"),
            Token::Constant(num) => write!(f, "{}", num),
            Token::Ident(ident) => write!(f, "{}", ident),
        }
    }
}

impl TryFrom<Token<'_>> for Register {
    type Error = String;

    fn try_from(value: Token) -> Result<Self, Self::Error> {
        match value {
            Token::Ident(ident) => ident.parse(),
            other => Err(format!("Expected register, found {other}")),
        }
    }
}

/// Peekable lexer that yields tokens one at a time.
pub struct Lexer<'a> {
    buf: &'a str,
    peek: Option<Token<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(buf: &'a str) -> Self {
        let mut lexer = Lexer { buf, peek: None };
        // Initalize the peek with the first token from the stream
        lexer.next();
        lexer
    }

    // Get the next token from the internal buffer.
    fn next_from_buf(&mut self) -> Option<Token<'a>> {
        self.buf = self.buf.trim();
        if self.buf.is_empty() {
            return None;
        }
        if let Some(rest) = self.buf.strip_prefix('(') {
            self.buf = rest;
            return Some(Token::LeftParen());
        } else if let Some(rest) = self.buf.strip_prefix(')') {
            self.buf = rest;
            return Some(Token::RightParen());
        } else if let Some(rest) = self.buf.strip_prefix(',') {
            self.buf = rest;
            return Some(Token::Comma());
        } else if let Some(rest) = self.buf.strip_prefix(':') {
            self.buf = rest;
            return Some(Token::Colon());
        } else if let Some(rest) = self.buf.strip_prefix('-') {
            self.buf = rest;
            return Some(Token::Minus());
        } else if let Some(rest) = self.buf.strip_prefix("0x") {
            if let Some((digits, rem)) = rest.consume(|c| c.is_ascii_hexdigit()) {
                return match parse_int::parse::<i32>(&format!("0x{}", digits)) {
                    Ok(number) => {
                        self.buf = rem;
                        Some(Token::Constant(number))
                    }
                    Err(_) => None,
                };
            } else {
                None
            }
        } else if let Some((digits, rest)) = self.buf.consume(|c| c.is_ascii_digit()) {
            return match parse_int::parse::<i32>(digits) {
                Ok(number) => {
                    self.buf = rest;
                    Some(Token::Constant(number))
                }
                Err(_) => None,
            };
        } else if let Some((label, rest)) = self.buf.consume(char::is_alphanumeric) {
            self.buf = rest;
            Some(Token::Ident(label))
        } else {
            None
        }
    }

    /// Return the next token, but don't advance the lexer.
    pub fn peek(&mut self) -> Option<Token<'a>> {
        self.peek
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;

    /// Return the next token and advance the lexer.
    fn next(&mut self) -> Option<Self::Item> {
        let old = self.peek;
        self.peek = self.next_from_buf();
        old
    }
}

/// Try to consume characters from a string that follow a certain predicate.
trait Consume<'a> {
    fn consume<F>(self, predicate: F) -> Option<(&'a str, &'a str)>
    where
        F: Fn(char) -> bool;
}

impl<'a> Consume<'a> for &'a str {
    fn consume<F>(self, predicate: F) -> Option<(&'a str, &'a str)>
    where
        F: Fn(char) -> bool,
    {
        self.char_indices()
            .take_while(|(_, c)| predicate(*c))
            .last()
            .map(|(index, _)| (&self[..index + 1], &self[index + 1..]))
    }
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
    lw { rd: Register, offset: i32, r1: Register  },
    sw { r2: Register, offset: i32, r1: Register },

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
    jal         { rd: Register, label: &'a str },
    jalr        { rd: Register, offset: i32, r1: Register },
    call        { label: &'a str },
    pseudo_jal  { label: &'a str },
    j           { label: &'a str },
    jr          { rs: Register },
    pseudo_jalr { rs: Register },
    ret         {},
}

/// An item of RISC-V assembly, either an instruction or label (for now)
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub enum Item<'a> {
    #[serde(borrow)]
    Instruction(Instruction<'a>),
    Label(&'a str),
}

pub fn decomment(source: &str) -> String {
    source
        .split('\n')
        .map(|s| {
            if let Some(index) = s.find('#') {
                &s[..index]
            } else {
                s
            }
        })
        .collect::<String>()
}

pub fn parse_item(source: &str) -> Result<Item, String> {
    let mut tokens = Lexer::new(source);
    let ident = tokens.ident()?.unwrap_ident();
    if let Ok(Token::Colon()) = tokens.colon() {
        return Ok(Item::Label(ident));
    }
    Ok(Item::Instruction(if REG_IMM.contains(&ident) {
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
            _ => unreachable!("list of REG_IMM opcodes does not match with match statement: you may have forgotten to keep them in sync")
        }
    } else if REG_REG.contains(&ident) {
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
            _ => unreachable!("list of REG_REG opcodes does not match with match statement: you may have forgotten to keep them in sync")
        }
    } else {
        todo!()
    }))
}

fn reg_imm_args(tokens: &mut Lexer) -> Result<(Register, Register, i32), String> {
    let rd = tokens.ident()?.try_into()?;
    let _ = tokens.comma()?;
    let r1 = tokens.ident()?.try_into()?;
    let _ = tokens.comma()?;
    let negative = tokens.minus().is_ok();
    let imm: i32 = tokens.constant()?.unwrap_constant();
    Ok((rd, r1, if negative { -imm } else { imm }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decomment_empty() {
        assert_eq!(decomment(""), "")
    }

    #[test]
    fn decomment_hashtag() {
        assert_eq!(decomment("#"), "")
    }

    #[test]
    fn decomment_single_line_comment() {
        assert_eq!(decomment("# blah blah"), "")
    }

    #[test]
    fn decomment_end_of_line_comment() {
        assert_eq!(decomment("add x0, x0, x0 # blah blah"), "add x0, x0, x0 ")
    }
}
