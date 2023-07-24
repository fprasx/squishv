use anyhow::bail;
use std::{fmt::Display, mem, ops::Range};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenInner<'a> {
    RightParen,
    LeftParen,
    Comma,
    Colon,
    Minus,
    Constant(i32),
    Ident(&'a str),
    Comment(&'a str),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    line: usize,
    columns: Range<usize>,
}

impl Span {
    pub fn new(line: usize, columns: Range<usize>) -> Self {
        Self { line, columns }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token<'a> {
    pub inner: TokenInner<'a>,
    span: Span,
}

// Functions for attempting to get a certain token from the lexer. If the next
// token is the desired token, the stream will be advanced and the token will be
// returned. Otherwise the stream is not advanced.
macro_rules! parse_token {
    ($($tokenpat:pat => $tokenfn:ident)+) => {
        $(
            impl<'a> LexerIter<'a> {
                pub fn $tokenfn(&mut self) -> ::anyhow::Result<Token<'a>> {
                    use ::anyhow::Context;
                    use $crate::lex::TokenInner::*;
                    match self.peek() {
                        // Unwrap is safe as we already peeked, we just use next
                        // to advance the stream
                        Some(Ok(Token { inner: $tokenpat, .. })) => self.next().unwrap(),
                        Some(Ok(Token { inner, .. })) => ::anyhow::bail!(
                            "Expected {}, found {inner}", stringify!($tokentype)
                        ),
                        Some(Err(_)) => self.next().unwrap().context(concat!("cannot parse ", stringify!($tokentype))),
                        None => ::anyhow::bail!(
                            "Expected {}, but ran out of input", stringify!($tokentype)
                        ),
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
    LeftParen => left_paren
    Minus => minus
    Ident(_) => ident
    Constant(_) => constant
    Comment(_) => comment
}

impl<'a> Token<'a> {
    pub fn new(inner: TokenInner<'a>, line: usize, columns: Range<usize>) -> Self {
        Token {
            inner,
            span: Span::new(line, columns),
        }
    }

    pub fn inner(&self) -> TokenInner<'a> {
        self.inner
    }

    /// Extract the string from an ident
    pub fn unwrap_ident(self) -> &'a str {
        match self.inner {
            TokenInner::Ident(inner) => inner,
            other => panic!("called unwrap ident on a {other}"),
        }
    }

    /// Extract the value of a constant
    pub fn unwrap_constant(self) -> i32 {
        match self.inner {
            TokenInner::Constant(inner) => inner,
            other => panic!("called unwrap ident on a {other}"),
        }
    }
}

impl Display for TokenInner<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenInner::RightParen => write!(f, "("),
            TokenInner::LeftParen => write!(f, ")"),
            TokenInner::Comma => write!(f, ","),
            TokenInner::Colon => write!(f, ":"),
            TokenInner::Minus => write!(f, "-"),
            TokenInner::Constant(num) => write!(f, "{}", num),
            TokenInner::Ident(ident) => write!(f, "{}", ident),
            TokenInner::Comment(ident) => write!(f, "'# {}'", ident),
        }
    }
}

type LexResult<'a> = anyhow::Result<Token<'a>>;

/// Low level lexer that can be turned into a peekable iterator over tokens.
pub struct Lexer<'a> {
    buf: &'a str,
    line: usize,
    char: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(buf: &'a str) -> Self {
        Lexer {
            buf,
            line: 1,
            char: 1,
        }
    }

    /// Gobble whitespace and update internal line/char positions
    fn gobble_whitespace(&mut self) {
        if let Some(spaces) = self.buf.consume(char::is_whitespace) {
            for space in spaces.chars() {
                if space == '\n' {
                    self.line += 1;
                    self.char = 1
                } else {
                    self.char += 1
                }
            }
            self.buf = &self.buf[spaces.len()..];
        }
    }

    /// Advances the lexer forward `token_len` characters. This means:
    ///
    /// - incrementing `self.char` by `token_len`
    /// - removing `token_len` characters form the front of `self.buf`
    ///
    /// *Note*: this function assumes there is no whitespace in the first
    /// `token_len` characters in `self.buf` and should only be called when
    /// it has been established that `self.buf[..token_len]` is a valid token.
    ///
    /// Returns the range `old self.char .. new self.char`, which is the span
    /// of the "token" we just made
    fn advance(&mut self, token_len: usize) -> Range<usize> {
        let span = self.char..self.char + token_len;
        self.char += token_len;
        self.buf = &self.buf[token_len..];
        span
    }

    // Parse another token
    fn next_from_buf(&mut self) -> Option<anyhow::Result<Token<'a>>> {
        self.gobble_whitespace();
        if self.buf.is_empty() {
            return None;
        }

        // These will be referenced very frequently :)
        let line = self.line;
        let start = self.char;

        // little utility for format errors with span info
        let fail = |error: &str, line, columns| bail!("{error}, line {line}, columns {columns:?}");

        Some(if self.buf.starts_with('(') {
            Ok(Token::new(TokenInner::LeftParen, line, self.advance(1)))
        } else if self.buf.starts_with(')') {
            Ok(Token::new(TokenInner::RightParen, line, self.advance(1)))
        } else if self.buf.starts_with(',') {
            Ok(Token::new(TokenInner::Comma, line, self.advance(1)))
        } else if self.buf.starts_with(':') {
            Ok(Token::new(TokenInner::Colon, line, self.advance(1)))
        } else if self.buf.starts_with('-') {
            Ok(Token::new(TokenInner::Minus, line, self.advance(1)))
        } else if let Some(rest) = self.buf.strip_prefix('#') {
            let text = rest.consume(|c| c != '\n').unwrap_or("");
            Ok(Token::new(
                TokenInner::Comment(text),
                line,
                // Add 1 for the '#'
                self.advance(text.len() + 1),
            ))
        } else if let Some(rest) = self.buf.strip_prefix("//") {
            let text = rest.consume(|c| c != '\n').unwrap_or("");
            Ok(Token::new(
                TokenInner::Comment(text),
                line,
                // Add 2 for the "//"
                self.advance(text.len() + 2),
            ))
        } else if let Some(rest) = self.buf.strip_prefix("0x") {
            // Note: parse hex literals before regular literals because we don't want
            // 0xabc to parse as Constant(0), Ident(xabc)
            if let Some(digits) = rest.consume(|c| c.is_ascii_hexdigit()) {
                let token_len = digits.len() + 2;
                let hex = format!("0x{}", digits);
                match parse_int::parse::<i32>(&hex) {
                    Ok(number) => Ok(Token::new(
                        TokenInner::Constant(number),
                        line,
                        self.advance(token_len),
                    )),
                    Err(_) => fail(
                        &format!("failed to parse '{hex}'"),
                        self.line,
                        self.char..self.char + token_len,
                    ),
                }
            } else {
                fail(
                    "got hex prefix 0x but no digits following",
                    self.line,
                    self.char..self.char + 2,
                )
            }
        } else if let Some(digits) = self.buf.consume(|c| c.is_ascii_digit()) {
            let token_len = digits.len();
            match parse_int::parse::<i32>(digits) {
                Ok(number) => Ok(Token::new(
                    TokenInner::Constant(number),
                    line,
                    self.advance(token_len),
                )),
                Err(_) => fail(
                    &format!("failed to parse {digits}"),
                    line,
                    start..start + token_len,
                ),
            }
        } else if let Some(label) = self.buf.consume(|c| c == '_' || c.is_alphanumeric()) {
            // Note: parse labels last as they can contain numbers, but we don't want
            // to parse 123 as a label
            Ok(Token::new(
                TokenInner::Ident(label),
                line,
                self.advance(label.len()),
            ))
        } else {
            fail(
                &format!("cannot parse '{}'", self.buf),
                line,
                start..start + 1,
            )
        })
    }
}

pub struct LexerIter<'a> {
    inner: Lexer<'a>,
    errored: bool,
    peek: Option<LexResult<'a>>,
}

impl<'a> LexerIter<'a> {
    pub fn peek(&mut self) -> Option<&LexResult<'a>> {
        self.peek.as_ref()
    }

    pub fn new(mut lexer: Lexer<'a>) -> Self {
        let next = lexer.next_from_buf();
        LexerIter { inner: lexer, errored: false, peek: next }
    }
}

impl<'a> Iterator for LexerIter<'a> {
    type Item = anyhow::Result<Token<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        if matches!(&self.peek, Some(Err(_))) {
            if !self.errored {
                self.errored = true;
                mem::replace(&mut self.peek, self.inner.next_from_buf())
            } else {
                None
            }
        } else {
            mem::replace(&mut self.peek, self.inner.next_from_buf())
        }
    }
}

impl<'a> IntoIterator for Lexer<'a> {
    type Item = anyhow::Result<Token<'a>>;

    type IntoIter = LexerIter<'a>;

    fn into_iter(mut self) -> Self::IntoIter {
        let peek = self.next_from_buf();
        LexerIter {
            inner: self,
            errored: false,
            peek,
        }
    }
}

/// Try to consume characters from a string that follow a certain predicate.
trait Consume<'a> {
    fn consume<F>(self, predicate: F) -> Option<&'a str>
    where
        F: Fn(char) -> bool;
}

impl<'a> Consume<'a> for &'a str {
    fn consume<F>(self, predicate: F) -> Option<&'a str>
    where
        F: Fn(char) -> bool,
    {
        self.char_indices()
            .take_while(|(_, c)| predicate(*c))
            .last()
            .map(|(index, _)| &self[..index + 1])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn consume_empty() {
        assert_eq!("".consume(|_| true), None)
    }

    #[test]
    fn consume_no_matches() {
        assert_eq!("abasalaka".consume(char::is_numeric), None)
    }

    #[test]
    fn consume_all_matches() {
        assert_eq!(
            "iamletters".consume(char::is_alphabetic),
            Some("iamletters",)
        )
    }

    #[test]
    fn consume_some_matches() {
        assert_eq!("LOUDsoft".consume(char::is_uppercase), Some("LOUD",))
    }

    #[test]
    fn lex_empty() {
        let mut lexer = Lexer::new("").into_iter();
        assert!(lexer.peek().is_none());
        assert!(lexer.next().is_none());
    }

    #[test]
    fn lex_error_returned_once() {
        let mut lexer = Lexer::new("&%^*").into_iter();
        assert!(matches!(lexer.next(), Some(Err(_))));
        assert!(lexer.next().is_none())
    }

    #[test]
    fn lex() {
        let tokens = Lexer::new(indoc! {"
            - , : ( )
            addi # and slli :)
            checka:
            loopa:
            69 -42 0xff
        "})
        .into_iter()
        .map(|token| token.unwrap())
        .collect::<Vec<_>>();
        assert_eq!(
            tokens,
            [
                Token::new(TokenInner::Minus, 1, 1..2),
                Token::new(TokenInner::Comma, 1, 3..4),
                Token::new(TokenInner::Colon, 1, 5..6),
                Token::new(TokenInner::LeftParen, 1, 7..8),
                Token::new(TokenInner::RightParen, 1, 9..10),
                Token::new(TokenInner::Ident("addi"), 2, 1..5),
                Token::new(TokenInner::Comment(" and slli :)"), 2, 6..19),
                Token::new(TokenInner::Ident("checka"), 3, 1..7),
                Token::new(TokenInner::Colon, 3, 7..8),
                Token::new(TokenInner::Ident("loopa"), 4, 1..6),
                Token::new(TokenInner::Colon, 4, 6..7),
                Token::new(TokenInner::Constant(69), 5, 1..3),
                Token::new(TokenInner::Minus, 5, 4..5),
                Token::new(TokenInner::Constant(42), 5, 5..7),
                Token::new(TokenInner::Constant(255), 5, 8..12),
            ]
        );
    }

    #[test]
    fn the_big_file() {
        assert!(Lexer::new(include_str!("test.s"))
            .into_iter()
            .all(|token| token.is_ok()))
    }

    #[test]
    fn gobble_whitespace() {
        let mut lexer = Lexer::new("");
        lexer.gobble_whitespace();
        assert_eq!(lexer.char, 1);
        assert_eq!(lexer.line, 1);

        let mut lexer = Lexer::new("   ");
        //                          123
        lexer.gobble_whitespace();
        assert_eq!(lexer.char, 4);
        assert_eq!(lexer.line, 1);

        let mut lexer = Lexer::new("\n\n");
        lexer.gobble_whitespace();
        assert_eq!(lexer.char, 1);
        assert_eq!(lexer.line, 3);

        let mut lexer = Lexer::new("   \n  \n    ");
        //                          123  12  1234
        lexer.gobble_whitespace();
        assert_eq!(lexer.char, 5);
        assert_eq!(lexer.line, 3);
    }
}
