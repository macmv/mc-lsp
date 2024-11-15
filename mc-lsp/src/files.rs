//! A virtual filesystem that tracks all the changes from the LSP client.

use std::{
  collections::HashMap,
  path::{Path, PathBuf},
};

use mc_source::FileId;

pub struct Files {
  /// Namespaces to absolute paths.
  namespace_roots: HashMap<Namespace, PathBuf>,

  files:       HashMap<FileId, File>,
  file_lookup: HashMap<FilePath, FileId>,

  changes: Vec<FileId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Namespace(String);

struct File {
  content: FileContent,
  path:    FilePath,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileContent {
  Json(String),
  Png(Vec<u8>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum FilePath {
  Rooted { namespace: Namespace, relative_path: PathBuf },
  // Some files don't have a source root, in which case we just leave this blank.
  Absolute(PathBuf),
}

impl Files {
  pub fn new() -> Self {
    Files {
      namespace_roots: HashMap::new(),
      files:           HashMap::new(),
      file_lookup:     HashMap::new(),
      changes:         vec![],
    }
  }

  pub fn add_namespace(&mut self, name: String, namespace_path: PathBuf) {
    let namespace = Namespace(name);
    self.namespace_roots.insert(namespace.clone(), namespace_path.clone());

    for file in self.files.values_mut() {
      if let FilePath::Absolute(path) = &mut file.path {
        if let Ok(relative) = path.strip_prefix(&namespace_path) {
          file.path =
            FilePath::Rooted { namespace: namespace.clone(), relative_path: relative.into() };
        }
      }
    }
  }

  fn make_file_path(&self, path: &Path) -> FilePath {
    assert!(path.is_absolute(), "cannot create source root for relative path {}", path.display());

    for (namespace, root) in &self.namespace_roots {
      if let Ok(relative) = path.strip_prefix(root) {
        return FilePath::Rooted {
          namespace:     namespace.clone(),
          relative_path: relative.to_path_buf(),
        };
      }
    }

    FilePath::Absolute(path.to_path_buf())
  }

  pub fn read(&self, id: FileId) -> FileContent {
    let file = self.files.get(&id).unwrap();
    file.content.clone()
  }
  pub fn write(&mut self, id: FileId, content: FileContent) {
    self.files.get_mut(&id).unwrap().content = content;
    self.changes.push(id);
  }

  pub fn take_changes(&mut self) -> Vec<FileId> { self.changes.drain(..).collect() }

  #[track_caller]
  pub fn create(&mut self, path: &Path) -> FileId {
    let path = self.make_file_path(path);
    let id = FileId::new_raw(self.files.len() as u32);

    self.file_lookup.insert(path.clone(), id);
    self.files.insert(id, File { content: FileContent::Json(String::new()), path });

    id
  }

  #[track_caller]
  pub fn get_absolute(&self, path: &Path) -> Option<FileId> {
    assert!(path.is_absolute(), "cannot lookup absolute for relative path {}", path.display());

    self.file_lookup.get(&self.make_file_path(path)).copied()
  }

  /// Returns `true` if the given file is in a namespace.
  pub fn in_namespace(&self, id: FileId) -> bool {
    match self.files[&id].path {
      FilePath::Rooted { .. } => true,
      FilePath::Absolute(_) => false,
    }
  }

  pub fn id_to_absolute_path(&self, id: FileId) -> PathBuf {
    let file = self.files.get(&id).unwrap();
    match &file.path {
      FilePath::Rooted { namespace, relative_path } => {
        self.namespace_roots[&namespace].join(relative_path)
      }
      FilePath::Absolute(path) => path.clone(),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn get_works() {
    let mut files = Files::new();
    let file = FileId::new_raw(0);

    files.add_namespace("foo".into(), "/foo".into());
    let id = files.create(Path::new("/foo/bar"));
    files.write(id, FileContent::Json("bar".to_string()));

    let id = files.get_absolute(Path::new("/foo/bar"));
    assert_eq!(id, Some(file));
  }

  #[test]
  fn get_works_with_no_root() {
    let mut files = Files::new();
    let file = FileId::new_raw(0);

    files.add_namespace("foo".into(), "/foo".into());
    let id = files.create(Path::new("/foo/bar"));
    files.write(id, FileContent::Json("bar".to_string()));

    let id = files.get_absolute(Path::new("/foo/bar"));
    assert_eq!(id, Some(file));
  }

  #[test]
  fn reindex_works() {
    let mut files = Files::new();

    let file_1 = files.create(Path::new("/foo/bar"));
    let file_2 = files.create(Path::new("/baz"));

    assert_eq!(files.files[&file_1].path, FilePath::Absolute(PathBuf::from("/foo/bar")));
    assert_eq!(files.files[&file_2].path, FilePath::Absolute(PathBuf::from("/baz")));

    files.add_namespace("foo".into(), "/foo".into());

    assert_eq!(
      files.files[&file_1].path,
      FilePath::Rooted {
        namespace:     Namespace("foo".into()),
        relative_path: PathBuf::from("bar"),
      }
    );
    assert_eq!(files.files[&file_2].path, FilePath::Absolute(PathBuf::from("/baz")));
  }
}
