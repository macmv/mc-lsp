pub mod highlight;

mod database;

#[allow(unused_imports)]
#[macro_use]
extern crate log;

use std::{panic::UnwindSafe, sync::Arc};

use database::{LineIndexDatabase, RootDatabase};
use highlight::Highlight;
use line_index::LineIndex;
use mc_hir::{model, HirDatabase};
use mc_source::{FileId, FileLocation, FileRange, SourceDatabase, Workspace};
use mc_syntax::{
  ast::{self, AstNode},
  AstPtr, T,
};
use salsa::{Cancelled, ParallelDatabase};

pub struct AnalysisHost {
  db: RootDatabase,
}

/// A snapshot of analysis at a point in time.
pub struct Analysis {
  db: salsa::Snapshot<RootDatabase>,
}

pub type Cancellable<T> = Result<T, Cancelled>;

impl Default for AnalysisHost {
  fn default() -> Self { Self::new() }
}

impl AnalysisHost {
  pub fn new() -> Self {
    let mut db = RootDatabase::default();
    db.set_workspace(Default::default());
    AnalysisHost { db }
  }

  pub fn snapshot(&self) -> Analysis { Analysis { db: self.db.snapshot() } }

  pub fn set_workspace(&mut self, workspace: mc_source::Workspace) {
    self.db.set_workspace(workspace.into());
  }

  pub fn add_file(&mut self, file: FileId) { self.db.set_file_text(file, "".into()); }

  pub fn workspace(&self) -> Arc<Workspace> { self.db.workspace() }

  pub fn change(&mut self, change: Change) {
    self.db.set_file_text(change.file, change.text.into());
  }
}

pub struct Change {
  pub file: FileId,
  pub text: String,
}

impl ParallelDatabase for RootDatabase {
  fn snapshot(&self) -> salsa::Snapshot<Self> {
    salsa::Snapshot::new(RootDatabase { storage: self.storage.snapshot() })
  }
}

impl Analysis {
  pub fn completions(&self, _: FileLocation) -> Cancellable<Vec<()>> { self.with_db(|_| vec![]) }
  pub fn diagnostics(&self, _: FileId) -> Cancellable<Vec<()>> { self.with_db(|_| vec![]) }

  pub fn highlight(&self, file: FileId) -> Cancellable<Highlight> {
    self.with_db(|db| Highlight::from_ast(db, file))
  }

  pub fn definition_for_name(&self, pos: FileLocation) -> Cancellable<Option<FileRange>> {
    self.with_db(|db| {
      let ast = db.parse_json(pos.file);
      let (model, source_map) = db.parse_model_with_source_map(pos.file);

      let token = ast
        .syntax_node()
        .token_at_offset(line_index::TextSize::from(pos.index.0))
        .max_by_key(|token| match token.kind() {
          T![string] => 10,
          T![number] => 9,

          _ => 1,
        })
        .unwrap();

      let nodes = token.parent_ancestors().filter_map(|node| match node.kind() {
        k if ast::Value::can_cast(k) => {
          let ptr = AstPtr::new(&ast::Value::cast(node).unwrap());
          source_map.ast_values.get(&ptr)
        }
        k if ast::Element::can_cast(k) => {
          let ptr = AstPtr::new(&ast::Element::cast(node).unwrap());
          source_map.ast_elements.get(&ptr)
        }
        _ => None,
      });

      for node in nodes {
        match model.nodes[*node] {
          model::Node::Texture(ref t) => {
            let name = match t {
              model::Texture::Reference(t) => t,
            };
            let node = model.texture_defs.iter().find_map(|id| {
              let model::Node::TextureDef(ref def) = model.nodes[*id] else { unreachable!() };

              if def.name == *name {
                Some(id)
              } else {
                None
              }
            });

            if let Some(node) = node {
              let element = source_map.texture_defs[&node].tree(&ast);

              return Some(FileRange {
                file:  pos.file,
                range: mc_source::TextRange {
                  start: mc_source::TextSize(element.syntax().text_range().start().into()),
                  end:   mc_source::TextSize(element.syntax().text_range().end().into()),
                },
              });
            }
          }
          model::Node::TextureDef(ref t) => {
            if t.value.starts_with("#") {
              continue;
            }

            let first = t.value.split(":").next();
            let second = t.value.split(":").nth(1);

            let (_namespace, _value) = match (first, second) {
              (Some(namespace), Some(value)) => (namespace, value),
              (Some(name), None) => ("minecraft", name),
              _ => continue,
            };

            dbg!(&t);
          }

          _ => {}
        }
      }

      None
    })
  }

  pub fn references_for_name(&self, _: FileLocation) -> Cancellable<Vec<FileRange>> {
    self.with_db(|_| vec![])
  }

  pub fn line_index(&self, file: FileId) -> Cancellable<Arc<LineIndex>> {
    self.with_db(|db| db.line_index(file))
  }

  fn with_db<T>(&self, f: impl FnOnce(&RootDatabase) -> T + UnwindSafe) -> Cancellable<T> {
    Cancelled::catch(|| f(&self.db))
  }
}
