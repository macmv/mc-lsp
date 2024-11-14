use mc_hir::{model, HirDatabase};
use mc_source::{FileLocation, Path, ResolvedPath};

#[derive(Debug, Clone)]
pub struct Completion {
  pub label: String,
}

struct Completer<'a> {
  db:    &'a dyn HirDatabase,
  pos:   FileLocation,
  model: &'a model::Model,

  completions: Vec<Completion>,
}

pub fn completions(db: &dyn HirDatabase, pos: FileLocation) -> Vec<Completion> {
  let Some(node) = db.node_at_index(pos) else { return vec![] };
  let model = db.parse_model(pos.file);

  let mut completer = Completer::new(db, pos, &model);

  match model.nodes[node] {
    model::Node::Parent(_) => {
      for n in db.workspace().namespaces.iter() {
        for f in n.files.iter() {
          if let Some(ResolvedPath::Model(path)) = f.path() {
            completer.complete_path(&path.path);
          }
        }
      }
    }

    model::Node::TextureDef(_) => {
      for n in db.workspace().namespaces.iter() {
        for f in n.files.iter() {
          if let Some(ResolvedPath::Texture(path)) = f.path() {
            completer.complete_path(&path.path);
          }
        }
      }
    }

    model::Node::Texture(_) => {
      for file in db.model_ancestry(pos.file) {
        let model = db.parse_model(file);
        for &def in model.texture_defs.iter() {
          let model::Node::TextureDef(ref def) = model.nodes[def] else { unreachable!() };

          completer.completions.push(Completion { label: format!("#{}", def.name.clone()) });
        }
      }
    }

    _ => {}
  }

  completer.completions
}

impl<'a> Completer<'a> {
  pub fn new(db: &'a dyn HirDatabase, pos: FileLocation, model: &'a model::Model) -> Completer<'a> {
    Completer { db, pos, model, completions: Vec::new() }
  }

  pub fn complete_path(&mut self, path: &Path) {
    self.completions.push(Completion { label: path.to_string() });
  }
}
