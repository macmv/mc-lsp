use mc_hir::{
  blockstate::{self, PropIter},
  model, HirDatabase,
};
use mc_source::{FileId, TextRange, TextSize};
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

  /// A model path.
  Model,

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

    match db.file_type(file) {
      mc_source::FileType::Model => hl.highlight_model(),
      mc_source::FileType::Blockstate => hl.highlight_blockstate(),
    }

    hl.hl.tokens.sort_by_key(|t| t.range.start());

    hl.hl
  }
}

impl Highlighter<'_> {
  fn new(db: &dyn HirDatabase, file: FileId) -> Highlighter {
    Highlighter { db, file, hl: Highlight { tokens: Vec::new() } }
  }

  fn highlight_model(&mut self) {
    let ast = self.db.parse_json(self.file);
    let (model, source_map, _) = self.db.parse_model_with_source_map(self.file);

    for (id, node) in model.nodes.iter() {
      match node {
        model::Node::TextureDef(_) => {
          let element = source_map.texture_defs[&id].tree(&ast);

          if let Some(key) = element.key() {
            self.highlight(key, HighlightKind::Variable);
          }
          if let Some(value) = element.value() {
            self.highlight(value, HighlightKind::Texture);
          }
        }
        model::Node::Texture(_) => {
          self.highlight(source_map.textures[&id].tree(&ast), HighlightKind::Variable);
        }
        model::Node::Parent(_) => {
          self.highlight(source_map.parent[&id].tree(&ast), HighlightKind::Model);
        }

        _ => {}
      }
    }
  }

  fn highlight_blockstate(&mut self) {
    let ast = self.db.parse_json(self.file);
    let (blockstate, source_map, _) = self.db.parse_blockstate_with_source_map(self.file);

    for (id, node) in blockstate.nodes.iter() {
      match node {
        blockstate::Node::Variant(ref v) => {
          let syntax = source_map.variants[&id].to_node(&ast);

          for (text, range) in PropIter::new(&v.name, &syntax) {
            let lhs = text.split('=').next().unwrap();
            let rhs = text.split('=').nth(1).unwrap_or("");

            self.highlight_range(
              TextRange::new(range.start(), range.start() + TextSize::from(lhs.len() as u32)),
              HighlightKind::Variable,
            );
            self.highlight_range(
              TextRange::new(range.end() - TextSize::from(rhs.len() as u32), range.end()),
              HighlightKind::Number,
            );
          }
        }

        blockstate::Node::Model(_) => {
          self.highlight(source_map.models[&id].tree(&ast), HighlightKind::Model);
        }
      }
    }
  }

  fn highlight<T: AstNode>(&mut self, node: T, kind: HighlightKind) {
    let range = node.syntax().text_range();
    self.highlight_range(range, kind);
  }

  fn highlight_range(&mut self, range: TextRange, kind: HighlightKind) {
    self.hl.tokens.push(HighlightToken { range, kind, modifierst: 0 });
  }
}

impl HighlightKind {
  pub fn iter() -> impl Iterator<Item = HighlightKind> {
    (0..=HighlightKind::Variable as u8).map(|i| unsafe { std::mem::transmute(i) })
  }
}
