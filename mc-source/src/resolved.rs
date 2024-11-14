use crate::Path;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResolvedPath {
  Model(ModelPath),
  Texture(TexturePath),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModelPath {
  pub path: Path,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TexturePath {
  pub path: Path,
}

impl ResolvedPath {
  pub fn parse(path: &Path) -> Option<Self> {
    if path.segments.get(0).is_some_and(|s| s == "models") {
      let mut path = path.clone();
      path.segments.remove(0);
      let last = path.segments.last_mut()?;
      *last = last.strip_suffix(".json")?.to_owned();
      Some(ResolvedPath::Model(ModelPath { path }))
    } else if path.segments.get(0).is_some_and(|s| s == "textures") {
      let mut path = path.clone();
      path.segments.remove(0);
      let last = path.segments.last_mut()?;
      *last = last.strip_suffix(".png")?.to_owned();
      Some(ResolvedPath::Texture(TexturePath { path }))
    } else {
      None
    }
  }
}

impl ModelPath {
  pub fn new(path: Path) -> Self { ModelPath { path } }

  pub fn file_path(&self) -> Path {
    let mut path = self.path.clone();
    path.segments.insert(0, "models".into());
    path.segments.last_mut().map(|s| s.push_str(".json"));
    path
  }
}

impl TexturePath {
  pub fn new(path: Path) -> Self { TexturePath { path } }

  pub fn file_path(&self) -> Path {
    let mut path = self.path.clone();
    path.segments.insert(0, "textures".into());
    path.segments.last_mut().map(|s| s.push_str(".png"));
    path
  }
}
