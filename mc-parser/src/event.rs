use std::mem;

use crate::SyntaxKind;

/// `Parser` produces a flat list of `Event`s.
/// They are converted to a tree-structure in
/// a separate pass, via `TreeBuilder`.
#[derive(Debug)]
pub enum Event {
  /// This event signifies the start of the node.
  /// It should be either abandoned (in which case the
  /// `kind` is `TOMBSTONE`, and the event is ignored),
  /// or completed via a `Finish` event.
  ///
  /// All tokens between a `Start` and a `Finish` would
  /// become the children of the respective node.
  ///
  /// For left-recursive syntactic constructs, the parser produces
  /// a child node before it sees a parent. `forward_parent`
  /// saves the position of current event's parent.
  ///
  /// Consider this path
  ///
  /// foo::bar
  ///
  /// The events for it would look like this:
  ///
  /// ```text
  /// START(PATH) IDENT('foo') FINISH START(PATH) T![::] IDENT('bar') FINISH
  ///       |                          /\
  ///       |                          |
  ///       +------forward-parent------+
  /// ```
  ///
  /// And the tree would look like this
  ///
  /// ```text
  ///    +--PATH---------+
  ///    |   |           |
  ///    |   |           |
  ///    |  '::'       'bar'
  ///    |
  ///   PATH
  ///    |
  ///   'foo'
  /// ```
  ///
  /// See also `CompletedMarker::precede`.
  Start {
    kind:           SyntaxKind,
    forward_parent: Option<u32>,
  },

  /// Complete the previous `Start` event
  Finish,

  /// Produce a single leaf-element.
  Token {
    kind: SyntaxKind,
    len:  usize,
  },

  Error {
    msg: String,
  },
}

impl Event {
  pub fn tombstone() -> Self { Event::Token { kind: SyntaxKind::TOMBSTONE, len: 0 } }
}

// Removes all the forward_parents. TODO: Produce a new Output type or something
// to avoid exposing Event.
pub fn process_events(events: &mut [Event]) -> Vec<Event> {
  let mut out = vec![];
  let mut forward_parents = Vec::new();

  for i in 0..events.len() {
    match mem::replace(&mut events[i], Event::tombstone()) {
      Event::Start { kind, forward_parent } => {
        // For events[A, B, C], B is A's forward_parent, C is B's forward_parent,
        // in the normal control flow, the parent-child relation: `A -> B -> C`,
        // while with the magic forward_parent, it writes: `C <- B <- A`.

        // append `A` into parents.
        forward_parents.push(kind);
        let mut idx = i;
        let mut fp = forward_parent;
        while let Some(fwd) = fp {
          idx += fwd as usize;
          // append `A`'s forward_parent `B`
          fp = match mem::replace(&mut events[idx], Event::tombstone()) {
            Event::Start { kind, forward_parent } => {
              forward_parents.push(kind);
              forward_parent
            }
            _ => unreachable!(),
          };
          // append `B`'s forward_parent `C` in the next stage.
        }

        for kind in forward_parents.drain(..).rev() {
          if kind != SyntaxKind::TOMBSTONE {
            out.push(Event::Start { kind, forward_parent: None });
          }
        }
      }
      Event::Finish => {
        out.push(Event::Finish);
      }
      Event::Token { kind, len } => {
        if kind != SyntaxKind::TOMBSTONE {
          out.push(Event::Token { kind, len });
        }
      }
      Event::Error { msg } => out.push(Event::Error { msg }),
    }
  }

  out
}

use std::fmt::{self, Debug};

pub fn format_events(events: &[Event], text: &str) -> String { Events(events, text).to_string() }
pub fn print_events(events: &[Event], text: &str) { println!("{}", Events(events, text)) }

struct Events<'a>(&'a [Event], &'a str);

impl fmt::Display for Events<'_> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut indent = 0;
    let mut index = 0;
    for e in self.0 {
      match e {
        Event::Start { kind, .. } => {
          writeln!(f, "{}{:?}", "  ".repeat(indent), kind)?;
          indent += 1;
        }
        Event::Finish => indent -= 1,
        Event::Token { kind, len } => {
          write!(f, "{}{:?}", "  ".repeat(indent), kind)?;
          if index >= self.1.len() {
            writeln!(f, " <EOF>")?;
            continue;
          }
          let str = &self.1[index..index + len];
          writeln!(f, " '{}'", str.replace('\n', "\\n"))?;
          index += len;
        }
        Event::Error { msg } => {
          writeln!(f, "{}error: {}", "  ".repeat(indent), msg)?;
        }
      }
    }
    Ok(())
  }
}
