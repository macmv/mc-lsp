use std::ops::Range;

use thiserror::Error;

use crate::{SyntaxKind, T};

pub type Result<T> = std::result::Result<T, LexError>;

#[derive(Debug, Error, PartialEq)]
pub enum LexError {
  #[error("invalid character")]
  InvalidChar,

  #[error("string terminated in newline")]
  NewlineInString,

  #[error("missing closing char quote")]
  MissingCharClose,

  #[error("end of file reached")]
  EOF,
}

// Below we have the lexer internals.

#[derive(Clone, Copy, Debug, PartialEq)]
enum InnerToken {
  Syntax(SyntaxKind),
  Text,
}

struct Tokenizer<'a> {
  source: &'a str,
  index:  usize,
}

impl<'a> Tokenizer<'a> {
  pub fn new(source: &'a str) -> Self { Tokenizer { source, index: 0 } }

  pub fn pos(&self) -> usize { self.index }

  pub fn peek(&mut self) -> Option<InnerToken> {
    if self.index >= self.source.len() {
      None
    } else {
      let chars = self.source[self.index..].chars().take(1);
      let t1 = self.eat();
      for c in chars {
        self.index -= c.len_utf8();
      }
      t1.ok()
    }
  }

  pub fn peek_char(&self) -> Option<char> { self.source[self.index..].chars().next() }
  pub fn peek_char2(&self) -> Option<char> { self.source[self.index..].chars().nth(1) }

  pub fn eat(&mut self) -> Result<InnerToken> {
    let Some(c) = self.source[self.index..].chars().next() else {
      return Err(LexError::EOF);
    };
    self.index += c.len_utf8();
    let t = match c {
      ' ' | '\t' | '\r' | '\n' => InnerToken::Syntax(SyntaxKind::WHITESPACE),

      '[' => InnerToken::Syntax(T!['[']),
      ']' => InnerToken::Syntax(T![']']),
      '{' => InnerToken::Syntax(T!['{']),
      '}' => InnerToken::Syntax(T!['}']),

      ':' => InnerToken::Syntax(T![:]),
      ',' => InnerToken::Syntax(T![,]),
      '\"' => InnerToken::Syntax(T!['"']),

      _ => InnerToken::Text,
    };
    Ok(t)
  }

  pub fn span(&self) -> Range<usize> { self.index - 1..self.index }
}

pub struct Lexer<'a> {
  tok:  Tokenizer<'a>,
  span: Range<usize>,

  pub in_string: bool,
}

impl<'a> Lexer<'a> {
  pub fn new(input: &'a str) -> Self {
    Lexer { tok: Tokenizer::new(input), span: 0..0, in_string: false }
  }

  fn ok(&mut self, start: usize, tok: SyntaxKind) -> Result<SyntaxKind> {
    self.span.start = start;
    self.span.end = self.tok.span().end;
    Ok(tok)
  }

  pub fn eat_whitespace(&mut self) -> Result<Option<SyntaxKind>> {
    loop {
      match self.tok.peek() {
        Some(InnerToken::Syntax(SyntaxKind::WHITESPACE)) => {
          self.tok.eat().unwrap();
          return Ok(Some(SyntaxKind::WHITESPACE));
        }
        Some(_) | None => break,
      }
    }
    Ok(None)
  }

  pub fn next(&mut self) -> Result<SyntaxKind> {
    let start = self.tok.pos();
    if let Some(t) = self.eat_whitespace()? {
      while self.eat_whitespace()?.is_some() {}
      return self.ok(start, t);
    }

    let char = self.tok.peek_char().ok_or(LexError::EOF)?;
    match char {
      // Numbers.
      '0'..='9' | '.' | '-' => {
        self.tok.eat()?;
        match self.tok.peek_char() {
          _ => {
            let mut is_float = false;
            loop {
              match self.tok.peek_char() {
                Some('0'..='9' | '_') => {}
                Some('.') => {
                  if !is_float && self.tok.peek_char2().is_some_and(|c| c.is_ascii_digit()) {
                    is_float = true;
                  } else {
                    break;
                  }
                }

                Some(_) | None => break,
              }

              self.tok.eat().unwrap();
            }

            self.ok(start, SyntaxKind::NUMBER)
          }
        }
      }

      // Strings.
      '"' => {
        self.tok.eat()?;
        let mut in_escape = false;
        loop {
          match self.tok.peek_char().ok_or_else(|| LexError::EOF)? {
            '\\' if in_escape => in_escape = false,

            _ if in_escape => in_escape = false,

            '\\' => in_escape = true,

            '"' => {
              self.tok.eat().unwrap();
              break self.ok(start, SyntaxKind::STRING);
            }

            _ => {}
          };

          self.tok.eat().unwrap();
        }
      }

      't' if self.tok.source[self.tok.index..].starts_with("true") => {
        self.tok.index += 4;
        self.ok(start, SyntaxKind::TRUE)
      }

      'f' if self.tok.source[self.tok.index..].starts_with("false") => {
        self.tok.index += 5;
        self.ok(start, SyntaxKind::FALSE)
      }

      'n' if self.tok.source[self.tok.index..].starts_with("null") => {
        self.tok.index += 4;
        self.ok(start, SyntaxKind::NULL)
      }

      _ => {
        self.tok.eat().unwrap();
        let kind = match char {
          '[' => T!['['],
          ']' => T![']'],
          '{' => T!['{'],
          '}' => T!['}'],

          ':' => T![:],
          ',' => T![,],
          '\"' => T!['"'],

          _ => return Err(LexError::InvalidChar),
        };

        self.ok(start, kind)
      }
    }
  }

  pub fn slice(&self) -> &'a str { &self.tok.source[self.span.clone()] }

  pub fn range(&self) -> Range<usize> { self.span.clone() }
  pub fn view(&self, range: Range<usize>) -> &'a str { &self.tok.source[range] }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn integers() {
    let mut lexer = Lexer::new("123");
    assert_eq!(lexer.next(), Ok(T![number]));
    assert_eq!(lexer.slice(), "123");
    assert_eq!(lexer.next(), Err(LexError::EOF));
  }

  #[test]
  fn floats() {
    let mut lexer = Lexer::new("2.345");
    assert_eq!(lexer.next(), Ok(T![number]));
    assert_eq!(lexer.slice(), "2.345");
    assert_eq!(lexer.next(), Err(LexError::EOF));

    let mut lexer = Lexer::new(".25");
    assert_eq!(lexer.next(), Ok(T![number]));
    assert_eq!(lexer.slice(), ".25");
    assert_eq!(lexer.next(), Err(LexError::EOF));
  }

  #[test]
  fn strings() {
    let mut lexer = Lexer::new("\"\"");
    assert_eq!(lexer.next(), Ok(T![string]));
    assert_eq!(lexer.slice(), "\"\"");
    assert_eq!(lexer.next(), Err(LexError::EOF));

    let mut lexer = Lexer::new(r#" "hi" "#);
    assert_eq!(lexer.next(), Ok(SyntaxKind::WHITESPACE));
    assert_eq!(lexer.slice(), " ");
    assert_eq!(lexer.next(), Ok(T![string]));
    assert_eq!(lexer.slice(), "\"hi\"");
    assert_eq!(lexer.next(), Ok(SyntaxKind::WHITESPACE));
    assert_eq!(lexer.slice(), " ");
    assert_eq!(lexer.next(), Err(LexError::EOF));

    let mut lexer = Lexer::new(
      r#" "hello
           world"
      "#,
    );
    assert_eq!(lexer.next(), Ok(SyntaxKind::WHITESPACE));
    assert_eq!(lexer.slice(), " ");
    assert_eq!(lexer.next(), Ok(T![string]));
    assert_eq!(lexer.slice(), "\"hello\n           world\"");
    assert_eq!(lexer.next(), Ok(SyntaxKind::WHITESPACE));
    assert_eq!(lexer.slice(), "\n      ");
    assert_eq!(lexer.next(), Err(LexError::EOF));

    // Escapes.
    let mut lexer = Lexer::new("\"foo: \\\"\"");
    assert_eq!(lexer.next(), Ok(T![string]));
    assert_eq!(lexer.slice(), "\"foo: \\\"\"");
    assert_eq!(lexer.next(), Err(LexError::EOF));

    let mut lexer = Lexer::new("\"\\\"\"");
    assert_eq!(lexer.next(), Ok(T![string]));
    assert_eq!(lexer.slice(), "\"\\\"\"");
    assert_eq!(lexer.next(), Err(LexError::EOF));
  }
}
