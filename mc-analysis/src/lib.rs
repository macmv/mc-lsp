pub mod completion;
pub mod highlight;

mod database;

#[cfg(test)]
mod tests;

#[allow(unused_imports)]
#[macro_use]
extern crate log;

use std::{collections::HashMap, panic::UnwindSafe, sync::Arc};

use completion::Completion;
use database::{LineIndexDatabase, RootDatabase};
use highlight::Highlight;
use line_index::LineIndex;
use mc_hir::{diagnostic::Diagnostics, model, HirDatabase};
use mc_source::{
  FileId, FileLocation, FileRange, FileType, SourceDatabase, TextRange, TextSize, Workspace,
};
use salsa::{Cancelled, ParallelDatabase};

pub use mc_hir::diagnostic;

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

  pub fn workspace(&self) -> Arc<Workspace> { self.db.workspace() }

  pub fn add_file(&mut self, file: FileId, ty: FileType, content: String) {
    self.db.set_file_type(file, ty);
    self.db.set_file_text(file, content.into());
  }
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
  pub fn completions(&self, pos: FileLocation) -> Cancellable<Vec<Completion>> {
    self.with_db(|db| completion::completions(db, pos))
  }
  pub fn diagnostics(&self, file: FileId) -> Cancellable<Diagnostics> {
    self.with_db(|db| {
      let mut diagnostics = Diagnostics::new();
      let parse = db.parse_json(file);
      for error in parse.errors() {
        diagnostics.error(
          TextRange::new(error.offset, error.offset + TextSize::new(1)),
          error.message.clone(),
        );
      }

      match db.file_type(file) {
        FileType::Model => diagnostics.extend(&db.validate_model(file)),
        FileType::Blockstate => diagnostics.extend(&db.validate_blockstate(file)),
      }

      diagnostics
    })
  }

  pub fn highlight(&self, file: FileId) -> Cancellable<Highlight> {
    self.with_db(|db| Highlight::from_ast(db, file))
  }

  pub fn definition_for_name(&self, pos: FileLocation) -> Cancellable<Option<FileRange>> {
    self.with_db(|db| match db.file_type(pos.file) {
      FileType::Model => db.model_def_at_index(pos),
      FileType::Blockstate => db.blockstate_def_at_index(pos),
    })
  }

  pub fn references_for_name(&self, _: FileLocation) -> Cancellable<Vec<FileRange>> {
    self.with_db(|_| vec![])
  }

  pub fn line_index(&self, file: FileId) -> Cancellable<Arc<LineIndex>> {
    self.with_db(|db| db.line_index(file))
  }

  pub fn canonical_model(&self, file: FileId) -> Cancellable<mc_message::Model> {
    self.with_db(|db| {
      let mut models = vec![];
      // Recurse child-up.
      let mut m = db.parse_model(file);
      loop {
        let Some(parent) = m.parent.as_ref() else { break };
        let Some(id) = db.lookup_model(parent.clone()) else { break };
        models.push(m);
        m = db.parse_model(id);
      }
      models.push(m);

      let mut model = mc_message::Model { elements: vec![] };

      // Recurse parent-down.
      for m in models.into_iter().rev() {
        // If this child defines elements, overwrite all of them.
        if m.nodes.values().any(|n| matches!(n, mc_hir::model::Node::Element(_))) {
          model.elements.clear();

          for node in m.nodes.values() {
            match node {
              mc_hir::model::Node::Element(e) => {
                model.elements.push(e.clone().into_hir(&m));
              }
              _ => {}
            }
          }
        }

        let mut texture_map = HashMap::<String, String>::new();
        for node in m.nodes.values() {
          match node {
            mc_hir::model::Node::TextureDef(ref def) => {
              texture_map.insert(def.name.clone(), def.value.clone());
            }
            _ => {}
          }
        }

        for element in model.elements.iter_mut() {
          for face in element.faces.iter_mut() {
            if let Some(ref tex) = face.texture {
              if let Some(name) = tex.strip_prefix("#") {
                if let Some(actual) = texture_map.get(name) {
                  face.texture = Some(actual.clone());
                }
              }
            }
          }
        }
      }

      // Remove any unlinked textures.
      for element in model.elements.iter_mut() {
        for face in element.faces.iter_mut() {
          if let Some(ref tex) = face.texture {
            if tex.starts_with("#") {
              face.texture = None;
            }
          }
        }
      }

      model
    })
  }

  fn with_db<T>(&self, f: impl FnOnce(&RootDatabase) -> T + UnwindSafe) -> Cancellable<T> {
    Cancelled::catch(|| f(&self.db))
  }
}

trait FromHir<T>
where
  Self: Sized,
{
  fn from_hir(hir: T, model: &model::Model) -> Self;
}
trait IntoHir<T>
where
  T: Sized,
{
  fn into_hir(self, model: &model::Model) -> T;
}
impl<T, U> IntoHir<U> for T
where
  U: FromHir<T>,
{
  fn into_hir(self, model: &model::Model) -> U { U::from_hir(self, model) }
}

impl FromHir<model::Element> for mc_message::Element {
  fn from_hir(hir: model::Element, model: &model::Model) -> Self {
    mc_message::Element {
      from:     hir.from.into_hir(model),
      to:       hir.to.into_hir(model),
      faces:    hir.faces.into_hir(model),
      rotation: None, // FIXME
    }
  }
}
impl FromHir<model::Faces> for mc_message::Faces {
  fn from_hir(hir: model::Faces, model: &model::Model) -> Self {
    macro_rules! face {
      ($face:expr) => {
        match $face {
          model::Node::Face(ref f) => f.clone(),
          _ => unreachable!(),
        }
      };
    }

    mc_message::Faces {
      north: hir.north.map(|n| face!(model.nodes[n]).into_hir(model)),
      east:  hir.east.map(|n| face!(model.nodes[n]).into_hir(model)),
      south: hir.south.map(|n| face!(model.nodes[n]).into_hir(model)),
      west:  hir.west.map(|n| face!(model.nodes[n]).into_hir(model)),
      up:    hir.up.map(|n| face!(model.nodes[n]).into_hir(model)),
      down:  hir.down.map(|n| face!(model.nodes[n]).into_hir(model)),
    }
  }
}
impl FromHir<model::Face> for mc_message::Face {
  fn from_hir(hir: model::Face, model: &model::Model) -> Self {
    mc_message::Face {
      uv:      [hir.uv[0].into(), hir.uv[1].into(), hir.uv[2].into(), hir.uv[3].into()],
      texture: match model.nodes[hir.texture] {
        model::Node::Texture(model::Texture::Reference(ref t)) => Some(format!("#{t}")),
        _ => unreachable!(),
      },
    }
  }
}

impl FromHir<model::Pos> for mc_message::Pos {
  fn from_hir(hir: model::Pos, _: &model::Model) -> Self {
    mc_message::Pos { x: hir.x.into(), y: hir.y.into(), z: hir.z.into() }
  }
}

impl<T: FromHir<U>, U> FromHir<Option<U>> for Option<T> {
  fn from_hir(hir: Option<U>, model: &model::Model) -> Self { hir.map(|hir| hir.into_hir(model)) }
}
