use std::{
  marker::PhantomData,
  ops::{Add, Sub},
  sync::Arc,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FileLocation {
  pub file:  FileId,
  pub index: TextSize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileRange {
  pub file:  FileId,
  pub range: TextRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TextSize(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextRange {
  pub start: TextSize,
  pub end:   TextSize,
}

impl Add for TextSize {
  type Output = TextSize;

  fn add(self, rhs: Self) -> Self::Output { TextSize(self.0 + rhs.0) }
}

impl Sub for TextSize {
  type Output = TextSize;

  fn sub(self, rhs: Self) -> Self::Output { TextSize(self.0 - rhs.0) }
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
  fn parse_model(&self, file_id: TypedFileId<Model>) -> <Model as FileType>::Source;
}

pub trait FileType {
  type Source;

  fn parse(text: &str) -> Self::Source;
}

pub struct TypedFileId<T: FileType> {
  pub raw:  FileId,
  _phantom: PhantomData<T>,
}

impl<T: FileType> Clone for TypedFileId<T> {
  fn clone(&self) -> Self { TypedFileId { raw: self.raw, _phantom: PhantomData } }
}
impl<T: FileType> Copy for TypedFileId<T> {}
impl<T: FileType> std::fmt::Debug for TypedFileId<T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "FileId<{:?}>({:?})", std::any::type_name::<T>(), self.raw)
  }
}
impl<T: FileType> PartialEq for TypedFileId<T> {
  fn eq(&self, other: &Self) -> bool { self.raw == other.raw }
}
impl<T: FileType> Eq for TypedFileId<T> {}
impl<T: FileType> std::hash::Hash for TypedFileId<T> {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) { self.raw.hash(state) }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FileId(u32);

impl<T: FileType> TypedFileId<T> {
  pub fn temp_new() -> Self { TypedFileId { raw: FileId(0), _phantom: PhantomData } }

  /// DO NOT USE THIS! Its just for unit tests.
  pub fn new_raw(id: u32) -> Self { TypedFileId { raw: FileId(id), _phantom: PhantomData } }
}

impl FileId {
  /// DO NOT USE THIS! Its just for unit tests.
  pub fn new_raw(id: u32) -> Self { FileId(id) }
}

#[derive(Default, Debug)]
pub struct Model;

impl FileType for Model {
  type Source = String;

  fn parse(text: &str) -> Self::Source { text.to_string() }
}

#[derive(Default, Debug)]
pub struct Workspace {
  pub files: Vec<FileId>,
}

fn parse<T: FileType>(db: &dyn SourceDatabase, file_id: FileId) -> T::Source {
  let text = db.file_text(file_id);
  T::parse(&text)
}

fn parse_model(
  db: &dyn SourceDatabase,
  file_id: TypedFileId<Model>,
) -> <Model as FileType>::Source {
  parse::<Model>(db, file_id.raw)
}
