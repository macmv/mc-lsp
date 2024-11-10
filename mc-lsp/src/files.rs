//! A virtual filesystem that tracks all the changes from the LSP client.

use std::{
  collections::HashMap,
  path::{Path, PathBuf},
};

use mc_source::FileId;

pub struct Files {
  files:       HashMap<FileId, File>,
  file_lookup: HashMap<FilePath, FileId>,

  root: PathBuf,

  changes: Vec<FileId>,
}

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
  Rooted { relative_path: PathBuf },
  // Some files don't have a source root, in which case we just leave this blank.
  Absolute(PathBuf),
}

impl Files {
  pub fn new(root: PathBuf) -> Self {
    Files { files: HashMap::new(), file_lookup: HashMap::new(), root, changes: vec![] }
  }

  fn make_file_path(&self, path: &Path) -> FilePath {
    assert!(path.is_absolute(), "cannot create source root for relative path {}", path.display());

    if let Ok(rel) = path.strip_prefix(&self.root) {
      FilePath::Rooted { relative_path: rel.to_path_buf() }
    } else {
      FilePath::Absolute(path.to_path_buf())
    }
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

    if self.within_root(path) {
      let relative = path.strip_prefix(&self.root).unwrap();

      self.file_lookup.get(&FilePath::Rooted { relative_path: relative.to_path_buf() }).copied()
    } else {
      self.file_lookup.get(&FilePath::Absolute(path.to_path_buf())).copied()
    }
  }

  pub fn relative_path<'a>(&self, path: &'a Path) -> Option<&'a Path> {
    assert!(path.is_absolute(), "cannot lookup relative for relative path {}", path.display());

    if self.within_root(path) {
      Some(path.strip_prefix(&self.root).unwrap())
    } else {
      None
    }
  }

  #[track_caller]
  fn within_root(&self, path: &Path) -> bool {
    assert!(path.is_absolute(), "cannot find source root for relative path {}", path.display());

    Self::within_root_lookup(&self.root, path)
  }

  fn within_root_lookup(root: &Path, path: &Path) -> bool {
    let mut p = path.to_path_buf();

    while p.pop() {
      if p == root {
        return true;
      }
    }

    false
  }

  pub fn id_to_absolute_path(&self, id: FileId) -> PathBuf {
    let file = self.files.get(&id).unwrap();
    match &file.path {
      FilePath::Rooted { relative_path } => self.root.join(relative_path),
      FilePath::Absolute(path) => path.clone(),
    }
  }

  pub fn change_root(&mut self, new_root: PathBuf) {
    self.file_lookup.clear();

    for file in self.files.values_mut() {
      let abs_path = match &file.path {
        FilePath::Rooted { relative_path } => self.root.join(relative_path),
        FilePath::Absolute(path) => path.clone(),
      };

      file.path = if Self::within_root_lookup(&new_root, &abs_path) {
        FilePath::Rooted { relative_path: abs_path.strip_prefix(&new_root).unwrap().to_path_buf() }
      } else {
        FilePath::Absolute(abs_path.clone())
      };
    }

    for (id, file) in &self.files {
      self.file_lookup.insert(file.path.clone(), *id);
    }

    self.root = new_root;
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn get_works() {
    let mut files = Files::new("/foo".into());
    let file = FileId::new_raw(0);

    let id = files.create(Path::new("/foo/bar"));
    files.write(id, FileContent::Json("bar".to_string()));

    let id = files.get_absolute(Path::new("/foo/bar"));
    assert_eq!(id, Some(file));
  }

  #[test]
  fn get_works_with_no_root() {
    let mut files = Files::new("/foo".into());
    let file = FileId::new_raw(0);

    let id = files.create(Path::new("/foo/bar"));
    files.write(id, FileContent::Json("bar".to_string()));

    let id = files.get_absolute(Path::new("/foo/bar"));
    assert_eq!(id, Some(file));
  }

  #[test]
  fn reindex_works() {
    let mut files = Files::new("/".into());

    let file_1 = files.create(Path::new("/foo/bar"));
    let file_2 = files.create(Path::new("/baz"));

    assert_eq!(files.files[&file_1].path, FilePath::Absolute(PathBuf::from("/foo/bar")));
    assert_eq!(files.files[&file_2].path, FilePath::Absolute(PathBuf::from("/baz")));

    files.change_root("/foo".into());

    assert_eq!(files.files[&file_1].path, FilePath::Rooted { relative_path: PathBuf::from("bar") });
    assert_eq!(files.files[&file_2].path, FilePath::Absolute(PathBuf::from("/baz")));
  }
}
