use std::{marker::PhantomData, sync::Arc};

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

  /// Parses the file into the syntax tree.
  fn parse_json(&self, file_id: FileId) -> Parse<mc_syntax::Json>;
}

pub trait FileType {
  type Source;

  fn parse(text: &str) -> Parse<Self::Source>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FileId(u32);

impl FileId {
  /// DO NOT USE THIS! Its just for unit tests.
  pub fn new_raw(id: u32) -> Self { FileId(id) }
}

#[derive(Default, Debug)]
pub struct Json;

impl FileType for Json {
  type Source = mc_syntax::Json;

  fn parse(text: &str) -> Parse<Self::Source> { mc_syntax::Json::parse(text) }
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
  pub path: Path,
}

impl File {
  pub fn path(&self) -> Option<ResolvedPath> { ResolvedPath::parse(&self.path) }
}

fn parse<T: FileType>(db: &dyn SourceDatabase, file_id: FileId) -> Parse<T::Source> {
  let text = db.file_text(file_id);
  T::parse(&text)
}

fn parse_json(db: &dyn SourceDatabase, file_id: FileId) -> Parse<<Json as FileType>::Source> {
  parse::<Json>(db, file_id)
}
