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
    ($( ($reg:ident = $xreg:ident) )*) => {
        impl FromStr for Register {
            type Err = &'static str;
            fn from_str(s: &str) -> Result<Register, &'static str> {
                match s.trim() {
                    $(
                        stringify!($reg) | stringify!($xreg)
                            => Ok(Register::$reg),
                    )*
                    "zero" => Ok(Register::x0),
                    _ => Err("unrecognized register")
                }
            }
        }
    }
}

register_parse_impl! {
   (x0 = x0)
   (ra = x1)
   (sp = x2)
   (gp = x3)
   (tp = x4)

   (t0 = x5)
   (t1 = x6)
   (t2 = x7)
   (t3 = x28)
   (t4 = x29)
   (t5 = x30)
   (t6 = x31)

   (a0 = x10)
   (a1 = x11)
   (a2 = x12)
   (a3 = x13)
   (a4 = x14)
   (a5 = x15)
   (a6 = x16)
   (a7 = x17)

   (s0 = x8)
   (s1 = x9)
   (s2 = x18)
   (s3 = x19)
   (s4 = x20)
   (s5 = x21)
   (s6 = x22)
   (s7 = x23)
   (s8 = x24)
   (s9 = x25)
   (s10 = x26)
   (s11 = x27)
}

#[derive(Debug, Clone, Copy)]
pub enum Token<'a> {
    RightParen,
    LeftParen,
    Comma,
    Colon,
    Constant(i32),
    Ident(&'a str),
}

pub trait Parse<T>
where
    Self: Sized,
{
    type ParseError;
    fn next(self) -> Result<(T, Self), Self::ParseError>;
}

enum LexerError {
    OutOfInput,
}

pub struct Lexer<'a> {
    buf: &'a str,
    tokens: Vec<Token<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(buf: &'a str) -> Self {
        Lexer {
            buf,
            tokens: vec![],
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.buf = self.buf.trim();
        if self.buf.is_empty() {
            return None;
        }
        if let Some(rest) = self.buf.strip_prefix('(') {
            self.buf = rest;
            return Some(Token::LeftParen);
        } else if let Some(rest) = self.buf.strip_prefix(')') {
            self.buf = rest;
            return Some(Token::RightParen);
        } else if let Some(rest) = self.buf.strip_prefix(',') {
            self.buf = rest;
            return Some(Token::Comma);
        } else if let Some(rest) = self.buf.strip_prefix(':') {
            self.buf = rest;
            return Some(Token::Colon);
        } else if let Some(rest) = self.buf.strip_prefix("0x") {
            if let Some((index, _)) = rest
                .char_indices()
                .take_while(|(_, c)| c.is_ascii_hexdigit())
                .last()
            {
                return match parse_int::parse::<i32>(&format!("0x{}", &rest[..index + 1])) {
                    Ok(number) => {
                        self.buf = &rest[index + 1..];
                        return Some(Token::Constant(number));
                    }
                    Err(_) => None,
                };
            }
        }
        if let Some((index, _)) = self
            .buf
            .char_indices()
            .take_while(|(_, c)| c.is_ascii_digit())
            .last()
        {
            match parse_int::parse::<i32>(&self.buf[..index + 1]) {
                Ok(number) => {
                    self.buf = &self.buf[index + 1..];
                    Some(Token::Constant(number))
                }

                Err(e) => None,
            }
        } else if let Some((index, _)) = self
            .buf
            .char_indices()
            .take_while(|(_, c)| c.is_alphanumeric())
            .last()
        {
            // Don't move this line before the next because it will then refer
            // to the next self.buf
            let ret = &self.buf[..index + 1];
            self.buf = &self.buf[index + 1..];
            Some(Token::Ident(ret))
        } else {
            None
        }
    }
}

trait Consume {
    fn consume<F>(&self, predicate: F) -> Option<(&str, &str)>
    where
        F: Fn(char) -> bool;
}

impl Consume for &str {
    fn consume<F>(&self, predicate: F) -> Option<(&str, &str)>
    where
        F: Fn(char) -> bool,
    {
        self.char_indices()
            .take_while(|(_, c)| predicate(*c))
            .last()
            .map(|(index, _)| (&self[..index + 1], &self[index + 1..]))
    }
}
