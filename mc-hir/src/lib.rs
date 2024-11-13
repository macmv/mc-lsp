use std::sync::Arc;

use diagnostic::Diagnostics;
use mc_source::{FileId, SourceDatabase};
use model::{Model, ModelPath};

pub mod diagnostic;
pub mod model;

#[allow(unused_imports)]
#[macro_use]
extern crate log;

#[salsa::query_group(HirDatabaseStorage)]
pub trait HirDatabase: SourceDatabase {
  #[salsa::invoke(model::parse_model)]
  fn parse_model_with_source_map(
    &self,
    file_id: FileId,
  ) -> (Arc<Model>, Arc<model::ModelSourceMap>, Arc<Diagnostics>);

  fn parse_model(&self, file_id: FileId) -> Arc<Model>;

  fn lookup_model(&self, path: ModelPath) -> Option<FileId>;
}

fn parse_model(db: &dyn HirDatabase, file_id: FileId) -> Arc<Model> {
  db.parse_model_with_source_map(file_id).0
}

fn lookup_model(db: &dyn HirDatabase, path: ModelPath) -> Option<FileId> {
  let workspace = db.workspace();
  let namespace = workspace.namespaces.iter().find(|n| n.name == path.path.namespace)?;

  // FIXME: This needs a lot of redoing.
  let mut search_path = path.path.clone();
  search_path.segments.insert(0, "models".into());
  *search_path.segments.last_mut().unwrap() += ".json";

  namespace.files.iter().find_map(|&(id, ref f)| if *f == search_path { Some(id) } else { None })
}
