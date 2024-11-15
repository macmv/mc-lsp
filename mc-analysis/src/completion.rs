use std::collections::HashSet;

use mc_hir::{blockstate, model, HirDatabase};
use mc_source::{FileLocation, FileType, Path, ResolvedPath};
use mc_syntax::{
  ast::{self, AstNode},
  SyntaxNode,
};

#[derive(Debug, Clone)]
pub struct Completion {
  pub label:       String,
  pub kind:        CompletionKind,
  pub description: String,

  pub retrigger: bool,
  pub insert:    String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionKind {
  Namespace,
  Model,
  Texture,
}

struct Completer {
  current_path: Option<PrefixPath>,

  completions: Vec<Completion>,
}

enum PrefixPath {
  // The cursor is in the last segment of the given path, and there is no namespace specified.
  NoNamespace(Vec<String>),
  // The cursor is in the last segment of the given path.
  Namespaced(Path),
}

pub fn completions(db: &dyn HirDatabase, pos: FileLocation) -> Vec<Completion> {
  match db.file_type(pos.file) {
    FileType::Model => model_completions(db, pos),
    FileType::Blockstate => blockstate_completions(db, pos),
  }
}

pub fn model_completions(db: &dyn HirDatabase, pos: FileLocation) -> Vec<Completion> {
  let keyword_completions = model_keyword_completions(db, pos);

  let Some(node) = db.model_node_at_index(pos) else { return keyword_completions };
  let model = db.parse_model(pos.file);

  let mut completer = Completer::new_model(db, pos, &model);
  completer.completions.extend(keyword_completions);

  match model.nodes[node] {
    model::Node::Parent(_) => {
      for n in db.workspace().namespaces.iter() {
        for f in n.files.iter() {
          if let Some(ResolvedPath::Model(path)) = f.resolved_path() {
            completer.complete_path(&path.path, CompletionKind::Model);
          }
        }
      }
    }

    model::Node::TextureDef(_) => {
      for n in db.workspace().namespaces.iter() {
        for f in n.files.iter() {
          if let Some(ResolvedPath::Texture(path)) = f.resolved_path() {
            completer.complete_path(&path.path, CompletionKind::Texture);
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
            kind:        CompletionKind::Texture,
            description: def.name.clone(),
            retrigger:   false,
            insert:      def.name.clone(),
          });
        }
      }
    }

    _ => {}
  }

  completer.completions
}

fn model_keyword_completions(db: &dyn HirDatabase, pos: FileLocation) -> Vec<Completion> {
  let token = mc_hir::token_at_offset(db, pos);

  let Some(parent) = token.parent() else { return vec![] };
  dbg!(&parent);

  // Exclude keys that have already been defined.
  let mut exclude = HashSet::new();
  if let Some(key) = ast::Key::cast(parent.clone()) {
    let element = ast::Element::cast(key.syntax().parent().unwrap()).unwrap();
    let obj = ast::Object::cast(element.syntax().parent().unwrap()).unwrap();
    for (key, _) in obj.iter() {
      exclude.insert(key.parse_text());
    }
  } else if let Some(obj) = ast::Object::cast(parent.clone()) {
    for (key, _) in obj.iter() {
      exclude.insert(key.parse_text());
    }
  }

  let mut path = vec![];
  for n in token.parent_ancestors() {
    if let Some(element) = ast::Element::cast(n) {
      if let Some(key) = element.key() {
        path.push(key.parse_text());
      }
    }
  }

  path.reverse();

  // Exclude the current key if we're completing within a key.
  if parent.kind() == ast::SyntaxKind::KEY {
    path.pop();
  }

  let strs = path.iter().map(|s| s.as_str()).collect::<Vec<_>>();
  let keywords: &[&str] = match strs.as_slice() {
    [] => &["parent", "textures", "elements"],
    ["elements"] => &["from", "to", "rotation", "faces"],
    ["elements", "faces"] => &["north", "south", "east", "west", "up", "down"],
    ["elements", "faces", _] => &["uv", "texture", "tintindex"],
    ["textures"] => &["particle"],
    _ => &[],
  };

  keywords
    .iter()
    .filter(|&&k| !exclude.contains(k))
    .map(|s| Completion {
      label:       format!("\"{}\"", s),
      kind:        CompletionKind::Namespace,
      description: s.to_string(),
      retrigger:   true,
      insert:      format!("\"{}\": ", s.to_string()),
    })
    .collect()
}

pub fn blockstate_completions(db: &dyn HirDatabase, pos: FileLocation) -> Vec<Completion> {
  let Some(node) = db.blockstate_node_at_index(pos) else { return vec![] };
  let blockstate = db.parse_blockstate(pos.file);

  let mut completer = Completer::new_blockstate(db, pos, &blockstate);

  match blockstate.nodes[node] {
    blockstate::Node::Model(_) => {
      for n in db.workspace().namespaces.iter() {
        for f in n.files.iter() {
          if let Some(ResolvedPath::Model(mut path)) = f.resolved_path() {
            // Blockstates implicitly add the 'block' element at the root of the path.
            path.path.segments.remove(0);
            completer.complete_path(&path.path, CompletionKind::Model);
          }
        }
      }
    }

    _ => {}
  }

  completer.completions
}

impl Completer {
  pub fn new_model(db: &dyn HirDatabase, pos: FileLocation, model: &model::Model) -> Completer {
    let node = db.model_node_at_index(pos).unwrap();
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

    Completer::new(current_path, pos)
  }

  pub fn new_blockstate(
    db: &dyn HirDatabase,
    pos: FileLocation,
    blockstate: &blockstate::Blockstate,
  ) -> Completer {
    let node = db.blockstate_node_at_index(pos).unwrap();
    let mut current_path = None;

    let ast = db.parse_json(pos.file);
    let (_, source_map, _) = db.parse_blockstate_with_source_map(pos.file);

    match blockstate.nodes[node] {
      blockstate::Node::Model(ref p) => {
        let node = source_map.models[&node].to_node(&ast);

        current_path = Some((p.path.clone(), node))
      }

      _ => {}
    }

    Completer::new(current_path, pos)
  }

  fn new(current_path: Option<(Path, SyntaxNode)>, pos: FileLocation) -> Completer {
    let current_path = current_path.map(|(_, node)| {
      // This is the location of the cursor within the path. The `-1` removes the
      // leading double quote.
      let offset = u32::from(pos.index - node.text_range().start()) as usize;

      // The part of the text to the left of the cursor. FIXME: Parse out escapes.
      let lhs = &node.text().to_string()[1..offset];

      if lhs.contains(":") {
        PrefixPath::Namespaced(lhs.to_string().parse().unwrap())
      } else {
        PrefixPath::NoNamespace(lhs.split('/').map(|s| s.to_string()).collect())
      }
    });

    Completer { current_path, completions: Vec::new() }
  }

  pub fn complete_path(&mut self, path: &Path, kind: CompletionKind) {
    match self.current_path {
      Some(PrefixPath::NoNamespace(ref segments)) => {
        if segments.len() == 1 {
          // If there is a single element, then just complete everything.
          self.completions.push(Completion {
            label: path.to_string(),
            kind,
            description: path.to_extended_string(),
            retrigger: false,
            insert: path.to_string(),
          });
        } else {
          // When there are segments, we are completing the path within the
          // "minecraft" namespace.
          let mut prefix = Path::new();
          prefix.segments = segments.clone();
          prefix.segments.pop();

          if let Some(to_complete) = path.strip_prefix(&prefix) {
            self.completions.push(Completion {
              label: to_complete.join("/"),
              kind,
              description: path.to_extended_string(),
              retrigger: false,
              insert: to_complete.join("/"),
            });
          }
        }
      }
      Some(PrefixPath::Namespaced(ref current)) => {
        let mut prefix = current.clone();
        prefix.segments.pop();

        if let Some(to_complete) = path.strip_prefix(&prefix) {
          self.completions.push(Completion {
            label: to_complete.join("/"),
            kind,
            description: path.to_extended_string(),
            retrigger: false,
            insert: to_complete.join("/"),
          });
        }
      }
      None => {}
    }
  }
}
