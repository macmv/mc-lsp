//! Converts files and a BSP workspace into FileIds and SourceRootIds.

use std::{io, path, path::PathBuf};

use mc_source::{FileId, Path};

use crate::files::{FileContent, Files};

pub fn discover_workspace(files: &mut Files) -> mc_source::Workspace {
  // We assume the root is the current directory. Then, we search for assets.

  let mut namespaces = vec![];

  let path = path::Path::new("./src/main/resources/assets");
  for entry in std::fs::read_dir(path).unwrap() {
    let name = entry.unwrap().file_name();

    let rel_path = mc_source::Path::new(name.to_string_lossy().to_string());

    let mut sources = vec![];
    let root_path: PathBuf = path.join(&name).canonicalize().unwrap();
    files.change_root(root_path.clone());
    discover_sources(&root_path, &rel_path, &mut sources, files).unwrap();

    namespaces
      .push(mc_source::Namespace { name: name.to_string_lossy().into_owned(), files: sources });
  }

  mc_source::Workspace { namespaces }
}

fn discover_sources(
  path: &path::Path,
  rel_path: &Path,
  sources: &mut Vec<(FileId, Path)>,
  files: &mut Files,
) -> io::Result<()> {
  for entry in std::fs::read_dir(path)? {
    let entry = entry?;
    let path = entry.path();
    if path.is_dir() {
      discover_sources(path.as_path(), &rel_path, sources, files)?;
    } else if let Some(r) = files.relative_path(&path) {
      let mut relative = Path::new(rel_path.namespace.clone());
      for segment in r.iter() {
        relative.segments.push(segment.to_string_lossy().to_string());
      }

      match files.get_absolute(&path) {
        Some(id) => {
          sources.push((id, relative.into()));
        }
        None => match r.extension() {
          Some(ext) if ext == "json" => {
            let id = files.create(&path);
            let content = std::fs::read_to_string(&path)?;
            files.write(id, FileContent::Json(content));
            sources.push((id, relative.into()));
          }
          Some(ext) if ext == "png" => {
            let id = files.create(&path);
            let content = std::fs::read(&path)?;
            files.write(id, FileContent::Png(content));
            sources.push((id, relative.into()));
          }
          _ => {}
        },
      }
    }
  }

  Ok(())
}
