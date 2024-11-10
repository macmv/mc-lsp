use mc_hir::HirDatabase;
use mc_source::{FileId, TextRange};
use mc_syntax::ast::AstNode;

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

  /// A texture path.
  Texture,

  /// Local variables.
  // Keep last!
  Variable,
}

#[allow(dead_code)]
struct Highlighter<'a> {
  db:   &'a dyn HirDatabase,
  file: FileId,

  hl: Highlight,
}

impl Highlight {
  pub fn from_ast(db: &dyn HirDatabase, file: FileId) -> Highlight {
    let mut hl = Highlighter::new(db, file);

    /*
    let ast = db.parse(file);

    // TODO
    let syntax = ast.syntax_node();
    for node in syntax.descendants() {}
    */

    let ast = db.parse_json(file);
    let (model, source_map) = db.parse_model_with_source_map(file);

    for texture in &model.textures {
      let element = source_map.textures[&texture].tree(&ast);

      if let Some(key) = element.key() {
        hl.highlight(key, HighlightKind::Variable);
      }
      if let Some(value) = element.value() {
        hl.highlight(value, HighlightKind::Texture);
      }
    }

    hl.hl.tokens.sort_by_key(|t| t.range.start);

    hl.hl
  }
}

impl Highlighter<'_> {
  fn new(db: &dyn HirDatabase, file: FileId) -> Highlighter {
    Highlighter { db, file, hl: Highlight { tokens: Vec::new() } }
  }

  fn highlight<T: AstNode>(&mut self, node: T, kind: HighlightKind) {
    let range = node.syntax().text_range();

    self.hl.tokens.push(HighlightToken {
      range: mc_source::TextRange {
        start: mc_source::TextSize(range.start().into()),
        end:   mc_source::TextSize(range.end().into()),
      },
      kind,
      modifierst: 0,
    });
  }
}

impl HighlightKind {
  pub fn iter() -> impl Iterator<Item = HighlightKind> {
    (0..=HighlightKind::Variable as u8).map(|i| unsafe { std::mem::transmute(i) })
  }
}
