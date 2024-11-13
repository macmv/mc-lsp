use std::sync::Arc;

use diagnostic::Diagnostics;
use mc_source::{FileId, FileLocation, FileRange, Path, SourceDatabase};
use mc_syntax::{
  ast::{self, AstNode},
  AstPtr, T,
};
use model::{Model, ModelPath};

pub mod diagnostic;
pub mod model;

#[allow(unused_imports)]
#[macro_use]
extern crate log;

#[salsa::query_group(HirDatabaseStorage)]
pub trait HirDatabase: SourceDatabase {
  #[salsa::invoke(model::parse_model)]
  fn parse_model_with_source_map(
    &self,
    file_id: FileId,
  ) -> (Arc<Model>, Arc<model::ModelSourceMap>, Arc<Diagnostics>);

  fn parse_model(&self, file_id: FileId) -> Arc<Model>;

  #[salsa::invoke(model::validate_model)]
  fn validate_model(&self, file_id: FileId) -> Arc<Diagnostics>;

  fn lookup_model(&self, path: ModelPath) -> Option<FileId>;

  fn def_at_index(&self, pos: FileLocation) -> Option<FileRange>;
  fn node_at_index(&self, pos: FileLocation) -> Option<model::NodeId>;
  fn def_at_node(&self, file: FileId, node: model::NodeId) -> Option<FileRange>;
}

fn parse_model(db: &dyn HirDatabase, file_id: FileId) -> Arc<Model> {
  db.parse_model_with_source_map(file_id).0
}

fn lookup_model(db: &dyn HirDatabase, path: ModelPath) -> Option<FileId> {
  let workspace = db.workspace();
  let namespace = workspace.namespaces.iter().find(|n| n.name == path.path.namespace)?;

  // FIXME: This needs a lot of redoing.
  let mut search_path = path.path.clone();
  search_path.segments.insert(0, "models".into());
  *search_path.segments.last_mut().unwrap() += ".json";

  namespace.files.iter().find_map(|&(id, ref f)| if *f == search_path { Some(id) } else { None })
}

fn node_at_index(db: &dyn HirDatabase, pos: FileLocation) -> Option<model::NodeId> {
  let ast = db.parse_json(pos.file);
  let (_, source_map, _) = db.parse_model_with_source_map(pos.file);

  let token = ast
    .syntax_node()
    .token_at_offset(pos.index)
    .max_by_key(|token| match token.kind() {
      T![string] => 10,
      T![number] => 9,

      _ => 1,
    })
    .unwrap();

  token.parent_ancestors().find_map(|node| match node.kind() {
    k if ast::Value::can_cast(k) => {
      let ptr = AstPtr::new(&ast::Value::cast(node).unwrap());
      source_map.ast_values.get(&ptr).copied()
    }
    k if ast::Element::can_cast(k) => {
      let ptr = AstPtr::new(&ast::Element::cast(node).unwrap());
      source_map.ast_elements.get(&ptr).copied()
    }
    _ => None,
  })
}

fn def_at_index(db: &dyn HirDatabase, pos: FileLocation) -> Option<FileRange> {
  let node = db.node_at_index(pos)?;

  db.def_at_node(pos.file, node)
}

fn def_at_node(db: &dyn HirDatabase, file: FileId, node: model::NodeId) -> Option<FileRange> {
  let model = db.parse_model(file);

  match model.nodes[node] {
    model::Node::Parent(ref p) => {
      let file = db.lookup_model(p.path.clone())?;

      Some(FileRange { file, range: None })
    }

    model::Node::Texture(ref t) => {
      let name = match t {
        model::Texture::Reference(t) => t,
      };
      let node = model.texture_defs.iter().find_map(|id| {
        let model::Node::TextureDef(ref def) = model.nodes[*id] else { unreachable!() };

        if def.name == *name {
          Some(id)
        } else {
          None
        }
      })?;

      // FIXME: Move this elsewhere, so `def_at_node` doesn't depend on the AST.
      let ast = db.parse_json(file);
      let (_, source_map, _) = db.parse_model_with_source_map(file);
      let element = source_map.texture_defs[&node].tree(&ast);

      Some(FileRange { file, range: Some(element.syntax().text_range()) })
    }
    model::Node::TextureDef(ref t) => {
      if t.value.starts_with("#") {
        return None;
      }

      let mut path: Path = t.value.parse().unwrap();

      path.segments.insert(0, "textures".into());
      *path.segments.last_mut().unwrap() += ".png";

      let workspace = db.workspace();
      let file = workspace.namespaces.iter().find_map(|n| {
        n.files.iter().find_map(|&(id, ref p)| if &path == p { Some(id) } else { None })
      })?;

      Some(FileRange { file, range: None })
    }

    _ => None,
  }
}
