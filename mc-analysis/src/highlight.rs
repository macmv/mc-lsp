use mc_source::{FileId, SourceDatabase, TextRange};
use mc_syntax::ast::SyntaxKind;

#[derive(Debug, Clone)]
pub struct Highlight {
  pub tokens: Vec<HighlightToken>,
}

#[derive(Debug, Clone)]
pub struct HighlightToken {
  pub range:      TextRange,
  pub kind:       HighlightKind,
  pub modifierst: u32,
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum HighlightKind {
  /// Special JSON keys.
  Keyword,

  /// Unkown JSON keys.
  UnknownKey,

  /// Numbers.
  Number,

  /// Booleans.
  Boolean,

  /// Null.
  Null,

  /// Local variables.
  // Keep last!
  Variable,
}

#[allow(dead_code)]
struct Highlighter<'a> {
  db:   &'a dyn SourceDatabase,
  file: FileId,

  hl: Highlight,
}

impl Highlight {
  pub fn from_ast(db: &dyn SourceDatabase, file: FileId) -> Highlight {
    let mut hl = Highlighter::new(db, file);

    /*
    let ast = db.parse(file);

    // TODO
    let syntax = ast.syntax_node();
    for node in syntax.descendants() {}
    */

    let tree = db.parse_json(file);

    for node in tree.syntax_node().descendants() {
      let range = node.text_range();

      let kind = match node.kind() {
        SyntaxKind::KEY => match node.text().to_string().as_str() {
          // FIXME: Need HIR!
          "\"parent\"" | "\"textures\"" => HighlightKind::Keyword,
          _ => HighlightKind::UnknownKey,
        },
        SyntaxKind::NUMBER => HighlightKind::Number,
        SyntaxKind::BOOLEAN => HighlightKind::Boolean,
        SyntaxKind::NULL => HighlightKind::Null,
        _ => continue,
      };

      hl.hl.tokens.push(HighlightToken {
        range: mc_source::TextRange {
          start: mc_source::TextSize(range.start().into()),
          end:   mc_source::TextSize(range.end().into()),
        },
        kind,
        modifierst: 0,
      });
    }

    hl.hl.tokens.sort_by_key(|t| t.range.start);

    hl.hl
  }
}

impl Highlighter<'_> {
  fn new(db: &dyn SourceDatabase, file: FileId) -> Highlighter {
    Highlighter { db, file, hl: Highlight { tokens: Vec::new() } }
  }
}

impl HighlightKind {
  pub fn iter() -> impl Iterator<Item = HighlightKind> {
    (0..=HighlightKind::Variable as u8).map(|i| unsafe { std::mem::transmute(i) })
  }
}
