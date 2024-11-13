use mc_hir::{model, HirDatabase};
use mc_source::FileLocation;

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
        n.files.iter().filter_map(|&(_, ref path)| {
          if path.segments.get(0).is_some_and(|s| s == "models")
            && path.segments.get(1).is_some_and(|s| s == "block")
          {
            let mut path = path.clone();
            // FIXME: Needs reworking!!
            path.segments.remove(0);
            path.segments.last_mut().map(|s| *s = s.strip_suffix(".json").unwrap_or(s).into());
            Some(Completion { label: path.to_string() })
          } else {
            None
          }
        })
      })
      .collect(),

    _ => vec![],
  }
}
