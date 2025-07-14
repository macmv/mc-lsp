//! Converts files and a BSP workspace into FileIds and SourceRootIds.

use std::{io, path, path::PathBuf};

use mc_source::{File, FileType, Path, Workspace};

use crate::files::{FileContent, Files};

pub fn discover_workspace(files: &mut Files) -> Workspace {
  // We assume the root is the current directory. Then, we search for assets.

  let mut workspace = Workspace { namespaces: vec![] };

  discover_assets_in(&mut workspace, files, path::Path::new("./src/main/resources/assets"));

  workspace
}

pub fn add_client_path(workspace: &mut Workspace, files: &mut Files, path: &path::Path) {
  discover_assets_in(workspace, files, &path.join("assets"));
}

fn discover_assets_in(workspace: &mut Workspace, files: &mut Files, path: &path::Path) {
  let dir = match std::fs::read_dir(path) {
    Ok(dir) => dir,
    Err(e) => {
      error!("failed to read directory {}: {}", path.display(), e);
      return;
    }
  };

  for entry in dir {
    let name = entry.unwrap().file_name();

    let rel_path = mc_source::Path::new_namespace(name.to_string_lossy().to_string());

    let mut sources = vec![];
    let root_path: PathBuf = path.join(&name).canonicalize().unwrap();
    files.add_namespace(rel_path.namespace.clone(), root_path.clone());
    discover_sources(&root_path, &rel_path, &mut sources, files).unwrap();

    workspace
      .namespaces
      .push(mc_source::Namespace { name: name.to_string_lossy().into_owned(), files: sources });
  }
}

fn discover_sources(
  path: &path::Path,
  relative: &Path,
  sources: &mut Vec<File>,
  files: &mut Files,
) -> io::Result<()> {
  for entry in std::fs::read_dir(path)? {
    let entry = entry?;
    let path = entry.path();

    let mut relative = relative.clone();
    relative.segments.push(path.file_name().unwrap().to_string_lossy().to_string());

    if path.is_dir() {
      discover_sources(path.as_path(), &relative, sources, files)?;
    } else {
      let ty = match relative.segments.first().map(|s| s.as_str()) {
        Some("models") => FileType::Model,
        Some("blockstates") => FileType::Blockstate,
        Some("textures") => FileType::Model, // FIXME: `ty` shouldn't exist on textures.
        _ => continue,
      };

      match files.get_absolute(&path) {
        Some(id) => {
          sources.push(File { id, ty, path: relative.clone() });
        }
        None => match path.extension() {
          Some(ext) if ext == "json" => {
            let id = files.create(&path);
            let content = std::fs::read_to_string(&path)?;
            files.write(id, FileContent::Json(content));
            sources.push(File { id, ty, path: relative });
          }
          Some(ext) if ext == "png" => {
            let id = files.create(&path);
            let content = std::fs::read(&path)?;
            files.write(id, FileContent::Png(content));
            // FIXME: This `ty` shouldn't exist on textures.
            sources.push(File { id, ty: FileType::Model, path: relative });
          }
          _ => {}
        },
      }
    }
  }

  Ok(())
}
