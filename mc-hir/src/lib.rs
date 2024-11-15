use std::sync::Arc;

use blockstate::Blockstate;
use diagnostic::Diagnostics;
use mc_source::{FileId, FileLocation, FileRange, ModelPath, ResolvedPath, SourceDatabase};
use mc_syntax::{
  ast::{self, AstNode},
  AstPtr, SyntaxToken, T,
};
use model::Model;

pub mod blockstate;
pub mod diagnostic;
pub mod model;
mod parse;

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

  #[salsa::invoke(blockstate::parse_blockstate)]
  fn parse_blockstate_with_source_map(
    &self,
    file_id: FileId,
  ) -> (Arc<Blockstate>, Arc<blockstate::BlockstateSourceMap>, Arc<Diagnostics>);

  fn parse_model(&self, file_id: FileId) -> Arc<Model>;
  fn parse_blockstate(&self, file_id: FileId) -> Arc<Blockstate>;

  #[salsa::invoke(model::validate_model)]
  fn validate_model(&self, file_id: FileId) -> Arc<Diagnostics>;

  #[salsa::invoke(blockstate::validate_blockstate)]
  fn validate_blockstate(&self, file_id: FileId) -> Arc<Diagnostics>;

  fn lookup_model(&self, path: ModelPath) -> Option<FileId>;

  /// Returns the ancestry, starting with the root, and ending with the child.
  #[salsa::invoke(model::ancestry)]
  fn model_ancestry(&self, file_id: FileId) -> Vec<FileId>;

  fn model_def_at_index(&self, pos: FileLocation) -> Option<FileRange>;
  fn model_node_at_index(&self, pos: FileLocation) -> Option<model::NodeId>;
  fn model_def_at_node(&self, file: FileId, node: model::NodeId) -> Option<FileRange>;

  fn blockstate_def_at_index(&self, pos: FileLocation) -> Option<FileRange>;
  fn blockstate_node_at_index(&self, pos: FileLocation) -> Option<blockstate::NodeId>;
  fn blockstate_def_at_node(&self, file: FileId, node: blockstate::NodeId) -> Option<FileRange>;
}

fn parse_model(db: &dyn HirDatabase, file_id: FileId) -> Arc<Model> {
  db.parse_model_with_source_map(file_id).0
}
fn parse_blockstate(db: &dyn HirDatabase, file_id: FileId) -> Arc<Blockstate> {
  db.parse_blockstate_with_source_map(file_id).0
}

fn lookup_model(db: &dyn HirDatabase, path: ModelPath) -> Option<FileId> {
  let workspace = db.workspace();
  let namespace = workspace.namespaces.iter().find(|n| n.name == path.path.namespace)?;

  let search_path = ResolvedPath::Model(path);

  namespace.files.iter().find_map(|f| {
    if f.resolved_path().as_ref() == Some(&search_path) {
      Some(f.id)
    } else {
      None
    }
  })
}

fn model_node_at_index(db: &dyn HirDatabase, pos: FileLocation) -> Option<model::NodeId> {
  let token = token_at_offset(db, pos);
  let (_, source_map, _) = db.parse_model_with_source_map(pos.file);

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

fn blockstate_node_at_index(db: &dyn HirDatabase, pos: FileLocation) -> Option<blockstate::NodeId> {
  let token = token_at_offset(db, pos);
  let (_, source_map, _) = db.parse_blockstate_with_source_map(pos.file);

  token.parent_ancestors().find_map(|node| match node.kind() {
    k if ast::Value::can_cast(k) => {
      let ptr = AstPtr::new(&ast::Value::cast(node).unwrap());
      source_map.ast_models.get(&ptr).copied()
    }
    _ => None,
  })
}

pub fn token_at_offset(db: &dyn HirDatabase, pos: FileLocation) -> SyntaxToken {
  let ast = db.parse_json(pos.file);

  ast
    .syntax_node()
    .token_at_offset(pos.index)
    .max_by_key(|token| match token.kind() {
      T![string] => 10,
      T![number] => 9,

      _ => 1,
    })
    .unwrap()
}

// FIXME: Dedupe with the model's version.
fn model_def_at_index(db: &dyn HirDatabase, pos: FileLocation) -> Option<FileRange> {
  let node = db.model_node_at_index(pos)?;

  db.model_def_at_node(pos.file, node)
}

fn blockstate_def_at_index(db: &dyn HirDatabase, pos: FileLocation) -> Option<FileRange> {
  let node = db.blockstate_node_at_index(pos)?;

  db.blockstate_def_at_node(pos.file, node)
}

fn model_def_at_node(db: &dyn HirDatabase, file: FileId, node: model::NodeId) -> Option<FileRange> {
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

      let search_path =
        ResolvedPath::Texture(mc_source::TexturePath::new(t.value.parse().unwrap()));

      let workspace = db.workspace();
      let file = workspace.namespaces.iter().find_map(|n| {
        n.files.iter().find_map(|f| {
          if f.resolved_path().as_ref() == Some(&search_path) {
            Some(f.id)
          } else {
            None
          }
        })
      })?;

      Some(FileRange { file, range: None })
    }

    _ => None,
  }
}

fn blockstate_def_at_node(
  db: &dyn HirDatabase,
  file: FileId,
  node: blockstate::NodeId,
) -> Option<FileRange> {
  let blockstate = db.parse_blockstate(file);

  match blockstate.nodes[node] {
    blockstate::Node::Model(ref p) => {
      let mut path = p.path.clone();
      // Blockstates implicitly have this at the start.
      path.segments.insert(0, "block".into());

      let file = db.lookup_model(ModelPath { path })?;

      Some(FileRange { file, range: None })
    }

    _ => None,
  }
}
