use mc_hir::{model, HirDatabase};
use mc_source::{FileLocation, ResolvedPath};

#[derive(Debug, Clone)]
pub struct Completion {
  pub label: String,
}

pub fn completions(db: &dyn HirDatabase, pos: FileLocation) -> Vec<Completion> {
  let Some(node) = db.node_at_index(pos) else { return vec![] };
  let model = db.parse_model(pos.file);

  match model.nodes[node] {
    model::Node::Parent(_) => db
      .workspace()
      .namespaces
      .iter()
      .flat_map(|n| {
        n.files.iter().filter_map(|f| match f.path() {
          Some(ResolvedPath::Model(path)) => Some(Completion { label: path.path.to_string() }),
          _ => None,
        })
      })
      .collect(),

    model::Node::TextureDef(_) => db
      .workspace()
      .namespaces
      .iter()
      .flat_map(|n| {
        n.files.iter().filter_map(|f| match f.path() {
          Some(ResolvedPath::Texture(path)) => Some(Completion { label: path.path.to_string() }),
          _ => None,
        })
      })
      .collect(),

    model::Node::Texture(_) => {
      let mut completions = vec![];

      for file in db.model_ancestry(pos.file) {
        let model = db.parse_model(file);
        for &def in model.texture_defs.iter() {
          let model::Node::TextureDef(ref def) = model.nodes[def] else { unreachable!() };

          completions.push(Completion { label: format!("#{}", def.name.clone()) });
        }
      }

      completions
    }

    _ => vec![],
  }
}
