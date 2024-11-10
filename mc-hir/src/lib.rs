use std::sync::Arc;

use mc_source::{FileId, SourceDatabase};
use model::Model;

mod model;

#[salsa::query_group(HirDatabaseStorage)]
pub trait HirDatabase: SourceDatabase {
  /// Returns the current content of the file.
  #[salsa::invoke(model::parse_model)]
  fn parse_model(&self, file_id: FileId) -> Arc<Model>;
}
