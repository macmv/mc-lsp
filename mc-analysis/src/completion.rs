use mc_hir::{model, HirDatabase};
use mc_source::{FileLocation, Path, ResolvedPath};
use mc_syntax::ast::{self, AstNode};

#[derive(Debug, Clone)]
pub struct Completion {
  pub label:       String,
  pub description: String,
}

struct Completer<'a> {
  #[allow(unused)]
  db:    &'a dyn HirDatabase,
  #[allow(unused)]
  pos:   FileLocation,
  #[allow(unused)]
  model: &'a model::Model,

  current_path: Option<PrefixPath>,

  completions: Vec<Completion>,
}

enum PrefixPath {
  // The cursor is in the namespace portion of the path.
  InNamespace,
  // The cursor is in the last segment of the given path.
  InPath(Path),
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

          completer.completions.push(Completion {
            label:       format!("#{}", def.name.clone()),
            description: def.name.clone(),
          });
        }
      }
    }

    _ => {}
  }

  completer.completions
}

impl<'a> Completer<'a> {
  pub fn new(db: &'a dyn HirDatabase, pos: FileLocation, model: &'a model::Model) -> Completer<'a> {
    let node = db.node_at_index(pos).unwrap();
    let mut current_path = None;

    let ast = db.parse_json(pos.file);
    let (_, source_map, _) = db.parse_model_with_source_map(pos.file);

    match model.nodes[node] {
      model::Node::Parent(ref p) => {
        let node = source_map.parent[&node].to_node(&ast);

        current_path = Some((p.path.path.clone(), node))
      }
      model::Node::TextureDef(ref p) => {
        let node = source_map.texture_defs[&node].to_node(&ast);
        let element = ast::Element::cast(node).unwrap();
        let value = element.value().unwrap();

        current_path = Some((p.value.parse().unwrap(), value.syntax().clone()))
      }

      _ => {}
    }

    let current_path = current_path.map(|(_, node)| {
      // This is the location of the cursor within the path. The `-1` removes the
      // leading double quote.
      let offset = u32::from(pos.index - node.text_range().start()) as usize;

      // The part of the text to the left of the cursor. FIXME: Parse out escapes.
      let lhs = &node.text().to_string()[1..offset];

      if lhs.contains(":") {
        let path = lhs.to_string().parse().unwrap();
        PrefixPath::InPath(path)
      } else {
        PrefixPath::InNamespace
      }
    });

    Completer { db, pos, model, current_path, completions: Vec::new() }
  }

  pub fn complete_path(&mut self, path: &Path) {
    match self.current_path {
      Some(PrefixPath::InNamespace) => {
        // This should be a small list, so performance is fine here.
        if !self.completions.iter().any(|c| c.label == path.namespace) {
          self.completions.push(Completion {
            label:       path.namespace.clone(),
            description: path.namespace.clone(),
          });
        }
      }
      Some(PrefixPath::InPath(ref current)) => {
        let mut prefix = current.clone();
        prefix.segments.pop();

        if let Some(to_complete) = path.strip_prefix(&prefix) {
          self
            .completions
            .push(Completion { label: to_complete.join("/"), description: path.to_string() });
        }
      }
      None => {}
    }
  }
}
