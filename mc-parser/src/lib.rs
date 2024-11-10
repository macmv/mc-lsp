mod event;
mod grammar;
mod syntax_kind;
#[cfg(test)]
mod tests;
mod token;

use std::ops::Range;

use drop_bomb::DropBomb;
use token::LexError;

pub use event::{format_events, print_events, process_events, Event};
pub use syntax_kind::SyntaxKind;
pub use token::Lexer;

#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

#[cfg(test)]
#[macro_use]
extern crate mc_test;

pub enum EntryPoint {
  Json,
}

struct Parser<'a> {
  lexer: &'a mut Lexer<'a>,

  current:            SyntaxKind,
  current_range:      Range<usize>,
  pending_whitespace: usize,
  peeked_whitespace:  usize,

  events: Vec<Event>,

  peeked: Option<(SyntaxKind, Range<usize>)>,

  // Tracks the current stack of braces. This is used to disabled newlines in areas surrounded by
  // `(` and `)`, or `[` and `]`.
  //
  // NB: Braces are not matched, so if the source file has mismatched braces, then this will stop
  // working! Additionally, this operates above `peek()`, so `peek()` will still incorrectly see
  // newline tokens.
  brace_stack: Vec<Brace>,

  // If set, then braces will not be matched.
  in_string: bool,
}

#[derive(Debug)]
enum Brace {
  /// `(` and `)`
  Paren,
  /// `[` and `]`
  Bracket,
  /// `{` and `}` (poorly named, I know).
  Brace,
  /// When in a pattern for a case item (manually added by the parser).
  Pattern,
}

impl EntryPoint {
  pub fn parse<'a>(&'a self, lexer: &'a mut Lexer<'a>) -> Vec<Event> {
    let mut parser = Parser::new(lexer);
    match self {
      EntryPoint::Json => grammar::entry_point::json(&mut parser),
    }
    parser.finish()
  }
}

impl<'a> Parser<'a> {
  pub fn new(lexer: &'a mut Lexer<'a>) -> Self {
    let mut p = Parser {
      current_range: 0..0,
      lexer,
      current: SyntaxKind::TOMBSTONE,
      events: Vec::new(),
      pending_whitespace: 0,
      peeked_whitespace: 0,
      peeked: None,
      in_string: false,
      brace_stack: vec![],
    };
    p.bump();
    p.events.clear(); // `bump` will push the current token, which we don't want here.
    p
  }
}

struct Marker {
  pos:  u32,
  bomb: DropBomb,
}

struct CompletedMarker {
  pos: u32,
}

impl Parser<'_> {
  pub fn finish(self) -> Vec<Event> {
    #[cfg(debug_assertions)]
    if !self.events.iter().any(|e| matches!(e, Event::Error { .. })) && !self.brace_stack.is_empty()
    {
      panic!("successful parse with non-empty brace stack: {:?}", self.brace_stack);
    }

    self.events
  }

  pub fn newlines_enabled(&self) -> bool {
    match self.brace_stack.last() {
      Some(Brace::Paren | Brace::Bracket | Brace::Pattern) => false,
      Some(Brace::Brace) | None => true,
    }
  }

  pub fn set_in_string(&mut self, in_string: bool) {
    self.in_string = in_string;
    self.lexer.in_string = in_string;
  }

  fn eat_trivia(&mut self) {
    if self.pending_whitespace > 0 {
      self
        .events
        .push(Event::Token { kind: SyntaxKind::WHITESPACE, len: self.pending_whitespace });
      self.pending_whitespace = 0;
    }
  }

  pub fn peek(&mut self) -> SyntaxKind {
    if let Some((p, _)) = self.peeked {
      p
    } else {
      // Don't push an even here. Instead, we'll push `current` when we consume the
      // peeked token in `bump`.
      let t = self.bump_peek();
      self.peeked = Some((t, self.lexer.range()));
      t
    }
  }

  pub fn start(&mut self) -> Marker {
    // Special case for the first marker: put whitespace after the start, so that we
    // get a single root node.
    if !self.events.is_empty() {
      self.eat_trivia();
    }

    let i = self.events.len() as u32;
    self.events.push(Event::Start { kind: SyntaxKind::TOMBSTONE, forward_parent: None });

    if self.events.len() == 1 {
      self.eat_trivia();
    }

    Marker { pos: i, bomb: DropBomb::new("Marker must be either completed or abandoned") }
  }
  pub fn at(&mut self, t: SyntaxKind) -> bool { self.current() == t }
  pub fn current(&self) -> SyntaxKind { self.current }
  pub fn slice(&self) -> &str { self.lexer.view(self.current_range.clone()) }
  #[track_caller]
  pub fn eat(&mut self, t: SyntaxKind) {
    assert_eq!(self.current(), t, "eat got unexpected result");
    self.bump();
  }
  pub fn bump(&mut self) -> SyntaxKind {
    if !self.in_string {
      match self.current {
        T!['['] => self.brace_stack.push(Brace::Bracket),
        T![']'] => {
          self.brace_stack.pop();
        }
        T!['{'] => self.brace_stack.push(Brace::Brace),
        T!['}'] => {
          self.brace_stack.pop();
        }
        _ => {}
      }
    }

    if let Some((t, r)) = self.peeked.take() {
      // Push `current`, now that we're pulling an event from `peeked`.
      self.eat_trivia();
      self.events.push(Event::Token { kind: self.current, len: self.current_range.len() });
      // TODO: Handle `semi` and `nl` correctly here.
      self.current = t;
      self.current_range = r;
      self.pending_whitespace = self.peeked_whitespace;
      self.peeked_whitespace = 0;
      t
    } else {
      let kind = self.bump_inner();
      self.current = kind;
      self.current_range = self.lexer.range();
      kind
    }
  }

  fn bump_inner(&mut self) -> SyntaxKind {
    self.eat_trivia();
    self.events.push(Event::Token { kind: self.current, len: self.lexer.slice().len() });

    loop {
      match self.lexer.next() {
        // Ignore whitespace tokens here, because we usually don't care about them when parsing. We
        // record that they got skipped, so that we can recover them later if we need a concrete
        // tree.
        Ok(SyntaxKind::WHITESPACE) if !self.in_string => {
          self.pending_whitespace += self.lexer.slice().len();
        }
        Ok(t) => break t,
        Err(LexError::EOF) => {
          break SyntaxKind::EOF;
        }
        Err(e) => {
          self.error(e.to_string());
          break self.current;
        }
      }
    }
  }

  fn bump_peek(&mut self) -> SyntaxKind {
    loop {
      match self.lexer.next() {
        // Ignore whitespace tokens here, because we usually don't care about them when parsing. We
        // record that they got skipped, so that we can recover them later if we need a concrete
        // tree.
        Ok(SyntaxKind::WHITESPACE) if !self.in_string => {
          self.peeked_whitespace += self.lexer.slice().len();
        }
        Ok(t) => break t,
        Err(LexError::EOF) => {
          break SyntaxKind::EOF;
        }
        Err(e) => {
          self.error(e.to_string());
          break self.current;
        }
      }
    }
  }

  pub fn expect(&mut self, t: SyntaxKind) {
    if self.current() != t {
      self.error(format!("expected {t:?}"));
    } else {
      self.bump();
    }
  }

  pub fn error(&mut self, msg: impl Into<String>) {
    self.events.push(Event::Error { msg: msg.into() })
  }
}

impl Marker {
  pub fn complete(mut self, parser: &mut Parser, kind: SyntaxKind) -> CompletedMarker {
    self.bomb.defuse();
    match &mut parser.events[self.pos as usize] {
      Event::Start { kind: k, .. } => *k = kind,
      _ => unreachable!(),
    }
    parser.events.push(Event::Finish);
    CompletedMarker { pos: self.pos }
  }
  pub fn abandon(mut self, parser: &mut Parser) {
    self.bomb.defuse();

    #[cfg(debug_assertions)]
    match parser.events[self.pos as usize] {
      Event::Start { kind: SyntaxKind::TOMBSTONE, forward_parent: None } => (),
      _ => unreachable!(),
    }

    if self.pos as usize == parser.events.len() - 1 {
      match parser.events.pop() {
        // Sanity check
        Some(Event::Start { kind: SyntaxKind::TOMBSTONE, forward_parent: None }) => (),
        _ => unreachable!(),
      }
    }
  }
}

impl CompletedMarker {
  fn precede(self, p: &mut Parser) -> Marker {
    let new_pos = p.start();
    let idx = self.pos as usize;
    match &mut p.events[idx] {
      Event::Start { forward_parent, .. } => {
        *forward_parent = Some(new_pos.pos - self.pos);
      }
      _ => unreachable!(),
    }
    new_pos
  }
}
