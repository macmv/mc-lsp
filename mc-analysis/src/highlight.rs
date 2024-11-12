use mc_hir::{model, HirDatabase};
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
    let (model, source_map, _) = db.parse_model_with_source_map(file);

    for (id, node) in model.nodes.iter() {
      match node {
        model::Node::TextureDef(_) => {
          let element = source_map.texture_defs[&id].tree(&ast);

          if let Some(key) = element.key() {
            hl.highlight(key, HighlightKind::Variable);
          }
          if let Some(value) = element.value() {
            hl.highlight(value, HighlightKind::Texture);
          }
        }
        model::Node::Texture(_) => {
          hl.highlight(source_map.textures[&id].tree(&ast), HighlightKind::Variable);
        }

        _ => {}
      }
    }

    hl.hl.tokens.sort_by_key(|t| t.range.start());

    hl.hl
  }
}

impl Highlighter<'_> {
  fn new(db: &dyn HirDatabase, file: FileId) -> Highlighter {
    Highlighter { db, file, hl: Highlight { tokens: Vec::new() } }
  }

  fn highlight<T: AstNode>(&mut self, node: T, kind: HighlightKind) {
    let range = node.syntax().text_range();

    self.hl.tokens.push(HighlightToken { range, kind, modifierst: 0 });
  }
}

impl HighlightKind {
  pub fn iter() -> impl Iterator<Item = HighlightKind> {
    (0..=HighlightKind::Variable as u8).map(|i| unsafe { std::mem::transmute(i) })
  }
}
