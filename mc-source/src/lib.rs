use std::sync::Arc;

use mc_syntax::Parse;

mod path;
mod resolved;

pub use line_index::{TextRange, TextSize};
pub use path::Path;
pub use resolved::{ModelPath, ResolvedPath, TexturePath};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FileLocation {
  pub file:  FileId,
  pub index: TextSize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileRange {
  pub file:  FileId,
  pub range: Option<TextRange>,
}

#[salsa::query_group(SourceDatabaseStorage)]
pub trait SourceDatabase: std::fmt::Debug {
  /// The current workspace.
  #[salsa::input]
  fn workspace(&self) -> Arc<Workspace>;

  /// Returns the current content of the file.
  #[salsa::input]
  fn file_text(&self, file_id: FileId) -> Arc<str>;

  #[salsa::input]
  fn file_type(&self, file_id: FileId) -> FileType;

  /// Parses the file into the syntax tree.
  fn parse_json(&self, file_id: FileId) -> Parse<mc_syntax::Json>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileType {
  Model,
  Blockstate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FileId(u32);

impl FileId {
  /// DO NOT USE THIS! Its just for unit tests.
  pub const fn new_raw(id: u32) -> Self { FileId(id) }
}

#[derive(Default, Debug)]
pub struct Workspace {
  pub namespaces: Vec<Namespace>,
}

#[derive(Default, Debug)]
pub struct Namespace {
  pub name: String,

  /// Files and their relative paths.
  pub files: Vec<File>,
}

#[derive(Debug)]
pub struct File {
  pub id:   FileId,
  pub ty:   FileType,
  pub path: Path,
}

impl File {
  pub fn resolved_path(&self) -> Option<ResolvedPath> { ResolvedPath::parse(&self.path) }
}

fn parse_json(db: &dyn SourceDatabase, file_id: FileId) -> Parse<mc_syntax::Json> {
  let text = db.file_text(file_id);
  mc_syntax::Json::parse(&text)
}
