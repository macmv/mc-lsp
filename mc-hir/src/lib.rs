use std::sync::Arc;

use mc_source::{FileId, SourceDatabase};
use model::Model;

pub mod model;

#[salsa::query_group(HirDatabaseStorage)]
pub trait HirDatabase: SourceDatabase {
  #[salsa::invoke(model::parse_model)]
  fn parse_model_with_source_map(
    &self,
    file_id: FileId,
  ) -> (Arc<Model>, Arc<model::ModelSourceMap>);

  fn parse_model(&self, file_id: FileId) -> Arc<Model>;
}

fn parse_model(db: &dyn HirDatabase, file_id: FileId) -> Arc<Model> {
  db.parse_model_with_source_map(file_id).0
}
