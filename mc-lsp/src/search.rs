//! Converts files and a BSP workspace into FileIds and SourceRootIds.

use std::{
  io,
  path::{Path, PathBuf},
};

use mc_source::FileId;

use crate::files::Files;

pub fn discover_workspace(files: &mut Files) -> mc_source::Workspace {
  // We assume the root is the current directory.
  let mut sources = vec![];
  let root_path: PathBuf = Path::new(".").canonicalize().unwrap();
  discover_sources(&root_path, &mut sources, files).unwrap();

  files.change_root(root_path);

  mc_source::Workspace { files: sources }
}

fn discover_sources(
  path: impl AsRef<Path>,
  sources: &mut Vec<(FileId, PathBuf)>,
  files: &mut Files,
) -> io::Result<()> {
  for entry in std::fs::read_dir(path)? {
    let entry = entry?;
    let path = entry.path();
    if path.is_dir() {
      let _ = discover_sources(&path, sources, files);
    } else {
      match files.get_absolute(&path) {
        Some(id) => {
          let relative = files.relative_path(&path).unwrap();
          sources.push((id, relative.into()));
        }
        None => {
          let relative = files.relative_path(&path).unwrap();
          let id = files.create(&path);
          let content = std::fs::read_to_string(&path)?;
          files.write(id, content);
          sources.push((id, relative.into()));
        }
      }
    }
  }

  Ok(())
}
