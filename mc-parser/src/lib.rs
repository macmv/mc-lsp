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

  events: Vec<Event>,
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
  #[allow(dead_code)]
  pos: u32,
}

impl Parser<'_> {
  pub fn finish(self) -> Vec<Event> { self.events }

  fn eat_trivia(&mut self) {
    if self.pending_whitespace > 0 {
      self
        .events
        .push(Event::Token { kind: SyntaxKind::WHITESPACE, len: self.pending_whitespace });
      self.pending_whitespace = 0;
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
  #[track_caller]
  pub fn eat(&mut self, t: SyntaxKind) {
    assert_eq!(self.current(), t, "eat got unexpected result");
    self.bump();
  }
  pub fn bump(&mut self) -> SyntaxKind {
    let kind = self.bump_inner();
    self.current = kind;
    self.current_range = self.lexer.range();
    kind
  }

  fn bump_inner(&mut self) -> SyntaxKind {
    self.eat_trivia();
    if self.current != SyntaxKind::__LAST {
      self.events.push(Event::Token { kind: self.current, len: self.lexer.slice().len() });
    }

    loop {
      match self.lexer.next() {
        // Ignore whitespace tokens here, because we usually don't care about them when parsing. We
        // record that they got skipped, so that we can recover them later if we need a concrete
        // tree.
        Ok(SyntaxKind::WHITESPACE) => {
          self.pending_whitespace += self.lexer.slice().len();
        }
        Ok(t) => break t,
        Err(LexError::EOF) => {
          break SyntaxKind::EOF;
        }
        Err(e) => {
          self.error(e.to_string());
          break SyntaxKind::__LAST;
        }
      }
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
  #[allow(dead_code)]
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
  #[allow(dead_code)]
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
