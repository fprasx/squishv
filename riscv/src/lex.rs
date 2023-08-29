use anyhow::{anyhow, Context};
use serde::{Deserialize, Serialize};
use std::{fmt, mem, ops::Range};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenInner {
    RightParen,
    LeftParen,
    Comma,
    Colon,
    Minus,
    Constant(i32),
    Ident(String),
    SlashComment(String),
    HashComment(String),
}

impl fmt::Display for TokenInner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenInner::RightParen => write!(f, "("),
            TokenInner::LeftParen => write!(f, ")"),
            TokenInner::Comma => write!(f, ","),
            TokenInner::Colon => write!(f, ":"),
            TokenInner::Minus => write!(f, "-"),
            TokenInner::Constant(num) => write!(f, "{}", num),
            TokenInner::Ident(ident) => write!(f, "{}", ident),
            TokenInner::HashComment(comment) => write!(f, "'# {}'", comment),
            TokenInner::SlashComment(comment) => write!(f, "'// {}'", comment),
        }
    }
}

/// Information for where a token occured in the source
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    line: usize,
    columns: Range<usize>,
}

impl Span {
    pub fn new(line: usize, columns: Range<usize>) -> Self {
        Self { line, columns }
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[line {}, columns {:?}]", self.line, self.columns)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub inner: TokenInner,
    span: Span,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let span = &self.span;
        match &self.inner {
            TokenInner::RightParen => {
                write!(f, ") {span}",)
            }
            TokenInner::LeftParen => {
                write!(f, "( {span}",)
            }
            TokenInner::Comma => write!(f, ", {span}",),
            TokenInner::Colon => write!(f, ": {span}",),
            TokenInner::Minus => write!(f, "- {span}",),
            TokenInner::Constant(val) => {
                write!(f, "{val} {span}",)
            }
            TokenInner::Ident(ident) => {
                write!(f, "'{ident}' {span}",)
            }
            TokenInner::HashComment(comment) => {
                write!(f, "'# {comment}' {span}",)
            }
            TokenInner::SlashComment(comment) => {
                write!(f, "'// {comment}' {span}",)
            }
        }
    }
}

// Functions for attempting to get a certain token from the lexer. If the next
// token is the desired token, the stream will be advanced and the token will be
// returned. Otherwise the stream is not advanced.
macro_rules! parse_token {
    ($($tokenpat:pat => $tokenfn:ident)+) => {
        // This function ensures that all variants are passed to the macro. If
        // a variant is not passed in, the match statement below will emit a
        // `all patterns not covered` error.
        #[doc(hidden)]
        fn __parse_token_ensure_all_variants(token: $crate::lex::Token) {
            use $crate::lex::TokenInner::*;
            match token.inner() {
                $(
                    $tokenpat => panic!("`__parse_token_ensure_all_variants` is not meant to be called"),
                 )*
            }
        }
        $(
            impl Lexer<'_> {
                pub fn $tokenfn(&mut self) -> ::anyhow::Result<Token> {
                    use $crate::lex::TokenInner::*;
                    match self.peek() {
                        // Unwrap is safe as we already peeked, we just use next
                        // to advance the stream
                        Some(Ok(Token { inner: $tokenpat, .. })) => self.next().unwrap(),
                        Some(Ok(Token { inner, span })) => ::anyhow::bail!(
                            "Expected {}, found {inner} at {span}", stringify!($tokenfn),
                        ),
                        Some(Err(e)) => {
                            // Note: _don't_ consume the error (via .next()), because it may
                            // be ok that the token isn't there. For example suppose we have
                            // the following situation parsing a number:
                            // -0x
                            // We might do something like:
                            // ```
                            // let neg = tokens.minus().is_ok();
                            // let num = tokens.constant()?.unwrap_constant().0;
                            // // do stuff with num and neg
                            // ```
                            // If calling .minus() on neg consumes the error, then
                            // the call with .constant() will just return "ran out of
                            // input" instead of the actual error, instead of the actual
                            // error, which is reported on the .minus() call.
                            ::anyhow::bail!("Expected {}, but {e}", stringify!($tokenfn))
                        }
                        None => ::anyhow::bail!(
                            "Expected {}, but ran out of input", stringify!($tokenfn)
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
    HashComment(_) => hash_comment
    SlashComment(_) => slash_comment
}

impl Token {
    pub fn new(inner: TokenInner, line: usize, columns: Range<usize>) -> Self {
        Token {
            inner,
            span: Span::new(line, columns),
        }
    }

    /// Extract the inner token
    pub fn inner(self) -> TokenInner {
        self.inner
    }

    /// Extract the span
    pub fn span(&self) -> Span {
        self.span.clone()
    }

    /// Split the token into the inner token and span
    pub fn split(self) -> (TokenInner, Span) {
        (self.inner, self.span)
    }

    /// Extract the string from an ident
    pub fn unwrap_ident(self) -> (String, Span) {
        match self.inner {
            TokenInner::Ident(inner) => (inner, self.span),
            other => panic!("called unwrap ident on a {other}"),
        }
    }

    /// Extract the value of a constant
    pub fn unwrap_constant(self) -> (i32, Span) {
        match self.inner {
            TokenInner::Constant(inner) => (inner, self.span),
            other => panic!("called unwrap ident on a {other}"),
        }
    }
}

type LexResult = anyhow::Result<Token>;

/// Low level lexer that can be turned into a peekable iterator over tokens.
/// There are a couple reasons this lexer should not be used directly:
///  - it is not peekable
///  - if it errors, it will just keep returning that error
/// The `Lexer` type wraps a `RawLexer` and takes cares of these things.
/// However, it is still nice to have actual lexing functionality abstracted
/// away in one place.
#[derive(Debug)]
struct RawLexer<'a> {
    buf: &'a str,
    line: usize,
    char: usize,
}

impl<'a> RawLexer<'a> {
    pub fn new(buf: &'a str) -> Self {
        RawLexer {
            buf,
            line: 1,
            char: 1,
        }
    }

    fn finished(&self) -> bool {
        self.buf.trim().is_empty()
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
    fn next_from_buf(&mut self) -> Option<anyhow::Result<Token>> {
        self.gobble_whitespace();
        if self.buf.is_empty() {
            return None;
        }

        // These will be referenced very frequently :)
        let line = self.line;
        let start = self.char;

        // little utility for format errors with span info
        let fail_message =
            |error: &str, line, columns| format!("{error}, line {line}, columns {columns:?}");

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
                TokenInner::HashComment(text.to_string()),
                line,
                // Add 1 for the '#'
                self.advance(text.len() + 1),
            ))
        } else if let Some(rest) = self.buf.strip_prefix("//") {
            let text = rest.consume(|c| c != '\n').unwrap_or("");
            Ok(Token::new(
                TokenInner::SlashComment(text.to_string()),
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

                // We cannot parse negative numbers into i32 (even if i32 can hold
                // negative numbers) because of "value out of bounds" errors. So
                // we first parse into a u32 then cast to i32.
                match parse_int::parse::<u32>(&hex) {
                    Ok(number) => Ok(Token::new(
                        TokenInner::Constant(number as i32),
                        line,
                        self.advance(token_len),
                    )),
                    Err(e) => Err(e).with_context(|| {
                        fail_message(
                            &format!("failed to parse '{hex}'"),
                            line,
                            start..start + token_len,
                        )
                    }),
                }
            } else {
                Err(anyhow!(fail_message(
                    "got hex prefix 0x but no digits following",
                    self.line,
                    self.char..self.char + 2,
                )))
            }
        } else if let Some(digits) = self.buf.consume(|c| c.is_ascii_digit()) {
            let token_len = digits.len();
            match parse_int::parse::<i32>(digits) {
                Ok(number) => Ok(Token::new(
                    TokenInner::Constant(number),
                    line,
                    self.advance(token_len),
                )),
                Err(e) => Err(e).with_context(|| {
                    fail_message(
                        &format!("failed to parse '{digits}'"),
                        line,
                        start..start + token_len,
                    )
                }),
            }
        } else if let Some(label) = self.buf.consume(|c| c == '_' || c.is_alphanumeric()) {
            // Note: parse labels last as they can contain numbers, but we don't want
            // to parse 123 as a label
            Ok(Token::new(
                TokenInner::Ident(label.to_string()),
                line,
                self.advance(label.len()),
            ))
        } else {
            Err(anyhow!(fail_message(
                &format!("cannot parse '{}'", self.buf),
                line,
                start..start + 1,
            )))
        })
    }
}

/// A peekable iterator over tokens. If an error is returned, the `Lexer` will
/// stop iteration afterwards.
#[derive(Debug)]
pub struct Lexer<'a> {
    inner: RawLexer<'a>,
    errored: bool,
    peek: Option<LexResult>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        let mut inner = RawLexer::new(source);
        let peek = inner.next_from_buf();
        Lexer {
            inner,
            errored: false,
            peek,
        }
    }

    pub fn peek(&mut self) -> Option<&LexResult> {
        self.peek.as_ref()
    }

    pub fn finished(&self) -> bool {
        self.inner.finished()
    }
}

impl Iterator for Lexer<'_> {
    type Item = anyhow::Result<Token>;

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

impl<'a> IntoIterator for RawLexer<'a> {
    type Item = anyhow::Result<Token>;

    type IntoIter = Lexer<'a>;

    fn into_iter(mut self) -> Self::IntoIter {
        let peek = self.next_from_buf();
        Lexer {
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
    fn empty() {
        let mut lexer = Lexer::new("");
        assert!(lexer.peek().is_none());
        assert!(lexer.next().is_none());
    }

    #[test]
    fn error_returned_once() {
        let mut lexer = Lexer::new("&%^*");
        assert!(matches!(lexer.next(), Some(Err(_))));
        assert!(lexer.next().is_none())
    }

    #[test]
    fn lex_negative() {
        let mut lexer = Lexer::new("0x80000000");
        assert_eq!(
            lexer.next().unwrap().unwrap(),
            Token::new(TokenInner::Constant(i32::MIN), 1, 1..11)
        )
    }

    /// This test asserts that peeking for a token with something like `.minus()`
    /// doesn't actually consume the internal error if it fails. So failing calls
    /// to `.minus()` are idempotent. Note that calling `next()` multiple times
    /// on the iterator will only return the error once. However, `.peek()` and its
    /// `.minus()` ilk actually have two semantic errors to return: incorrect token,
    /// or error lexing a token. For now, we just go with the first, and allow
    /// a call to `.minus()` to fail several times, intead of mimicking `.next()`'s
    /// behaviour, which only fails once.
    #[test]
    fn idempotent_peek() {
        let mut lexer = Lexer::new("0x");
        // this will attempt to parse a token, which will cause an error,
        // however, it does not consume the error
        assert!(lexer.minus().is_err());
        // error not consumed
        assert!(lexer.peek().is_some());
        // do it a couple more times for good measure
        for _ in 0..10 {
            assert!(lexer.minus().is_err());
            assert!(lexer.peek().is_some());
        }
    }

    #[test]
    fn lex() {
        let tokens = Lexer::new(indoc! {"
            - , : ( )
            addi # and slli :)
            // slashy slash
            checka:
            loopa:
            69 -42 0xff
        "})
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
                Token::new(TokenInner::Ident("addi".to_string()), 2, 1..5),
                Token::new(
                    TokenInner::HashComment(" and slli :)".to_string()),
                    2,
                    6..19
                ),
                Token::new(
                    TokenInner::SlashComment(" slashy slash".to_string()),
                    3,
                    1..16
                ),
                Token::new(TokenInner::Ident("checka".to_string()), 4, 1..7),
                Token::new(TokenInner::Colon, 4, 7..8),
                Token::new(TokenInner::Ident("loopa".to_string()), 5, 1..6),
                Token::new(TokenInner::Colon, 5, 6..7),
                Token::new(TokenInner::Constant(69), 6, 1..3),
                Token::new(TokenInner::Minus, 6, 4..5),
                Token::new(TokenInner::Constant(42), 6, 5..7),
                Token::new(TokenInner::Constant(255), 6, 8..12),
            ]
        );
    }

    #[test]
    fn fuzz() {
        assert!(Lexer::new(include_str!("../tests/test.s")).all(|token| token.is_ok()));
        assert!(Lexer::new(include_str!("../tests/random.s")).all(|token| token.is_ok()));
    }

    #[test]
    fn gobble_whitespace() {
        let mut lexer = RawLexer::new("");
        lexer.gobble_whitespace();
        assert_eq!(lexer.char, 1);
        assert_eq!(lexer.line, 1);

        let mut lexer = RawLexer::new("   ");
        //                          123
        lexer.gobble_whitespace();
        assert_eq!(lexer.char, 4);
        assert_eq!(lexer.line, 1);

        let mut lexer = RawLexer::new("\n\n");
        lexer.gobble_whitespace();
        assert_eq!(lexer.char, 1);
        assert_eq!(lexer.line, 3);

        let mut lexer = RawLexer::new("   \n  \n    ");
        //                          123  12  1234
        lexer.gobble_whitespace();
        assert_eq!(lexer.char, 5);
        assert_eq!(lexer.line, 3);
    }
}
