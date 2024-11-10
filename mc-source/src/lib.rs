use std::{collections::HashSet, marker::PhantomData, path::PathBuf, sync::Arc};

use la_arena::{Arena, Idx};
use url::Url;

mod source_root;

#[salsa::query_group(SourceDatabaseStorage)]
pub trait SourceDatabase: std::fmt::Debug {
  /// The current workspace.
  #[salsa::input]
  fn workspace(&self) -> Arc<Workspace>;

  /// Returns the current content of the file.
  #[salsa::input]
  fn file_text(&self, file_id: RawFileId) -> Arc<str>;

  /// Parses the file into the syntax tree.
  fn parse_model(&self, file_id: FileId<Model>) -> <Model as FileType>::Source;

  #[salsa::input]
  fn file_source_root(&self, file_id: RawFileId) -> Option<SourceRootId>;

  #[salsa::invoke(source_root::source_root_target)]
  fn source_root_target(&self, id: SourceRootId) -> TargetId;

  #[salsa::invoke(source_root::file_target)]
  fn file_target(&self, file_id: RawFileId) -> Option<TargetId>;
}

pub trait FileType {
  type Source;

  fn parse(text: &str) -> Self::Source;
}

pub struct FileId<T: FileType> {
  raw:      RawFileId,
  _phantom: PhantomData<T>,
}

impl<T: FileType> Clone for FileId<T> {
  fn clone(&self) -> Self { FileId { raw: self.raw, _phantom: PhantomData } }
}
impl<T: FileType> Copy for FileId<T> {}
impl<T: FileType> std::fmt::Debug for FileId<T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "FileId<{:?}>({:?})", std::any::type_name::<T>(), self.raw)
  }
}
impl<T: FileType> PartialEq for FileId<T> {
  fn eq(&self, other: &Self) -> bool { self.raw == other.raw }
}
impl<T: FileType> Eq for FileId<T> {}
impl<T: FileType> std::hash::Hash for FileId<T> {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) { self.raw.hash(state) }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RawFileId(u32);

impl<T: FileType> FileId<T> {
  pub fn temp_new() -> Self { FileId { raw: RawFileId(0), _phantom: PhantomData } }

  /// DO NOT USE THIS! Its just for unit tests.
  pub fn new_raw(id: u32) -> Self { FileId { raw: RawFileId(id), _phantom: PhantomData } }
}

pub struct Model;

impl FileType for Model {
  type Source = String;

  fn parse(text: &str) -> Self::Source { text.to_string() }
}

#[derive(Default, Debug)]
pub struct Workspace {
  pub root: PathBuf,

  pub targets:      Arena<TargetData>,
  pub source_roots: Arena<SourceRoot>,
}

impl Workspace {
  pub fn all_dependencies(&self, target: TargetId) -> DependencyIter {
    DependencyIter { workspace: self, seen: HashSet::new(), stack: vec![target] }
  }
}

pub struct DependencyIter<'a> {
  workspace: &'a Workspace,
  seen:      HashSet<TargetId>,
  stack:     Vec<TargetId>,
}

impl Iterator for DependencyIter<'_> {
  type Item = TargetId;

  fn next(&mut self) -> Option<Self::Item> {
    let mut target = self.stack.pop()?;
    while !self.seen.insert(target) {
      target = self.stack.pop()?;
    }
    self.stack.extend(self.workspace.targets[target].dependencies.iter().copied());
    Some(target)
  }
}

/// Targets are similar to packages, but are slightly more granular. For
/// example, one project may have a target for its main sources, and a target
/// for its test sources.
///
/// Target sources are unique to each target.
#[derive(Debug)]
pub struct TargetData {
  pub dependencies: Vec<TargetId>,

  pub bsp_id: Url,

  /// A list of directories which contain the source files for this target.
  pub source_roots: Vec<SourceRootId>,
}

pub type TargetId = Idx<TargetData>;

#[derive(Debug)]
pub struct SourceRoot {
  pub path:    PathBuf,
  pub sources: Vec<RawFileId>,
}

pub type SourceRootId = Idx<SourceRoot>;

fn parse<T: FileType>(db: &dyn SourceDatabase, file_id: RawFileId) -> T::Source {
  let text = db.file_text(file_id);
  T::parse(&text)
}

fn parse_model(db: &dyn SourceDatabase, file_id: FileId<Model>) -> <Model as FileType>::Source {
  parse::<Model>(db, file_id.raw)
}
