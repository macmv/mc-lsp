use std::{collections::HashMap, sync::Arc};

mod parse;
mod validate;

use la_arena::{Arena, Idx};
use mc_source::{FileId, Path};
use mc_syntax::{ast, AstPtr};

use crate::{diagnostic::Diagnostics, model::F64Eq, HirDatabase};

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Blockstate {
  pub nodes: Arena<Node>,
}

pub type NodeId = Idx<Node>;

#[derive(Debug, PartialEq, Eq)]
pub enum Node {
  Variant(Variant),
  Model(Model),
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct BlockstateSourceMap {
  pub ast_variants: HashMap<AstPtr<ast::Element>, NodeId>,
  pub ast_models:   HashMap<AstPtr<ast::Value>, NodeId>,

  pub variants: HashMap<NodeId, AstPtr<ast::Element>>,
  pub models:   HashMap<NodeId, AstPtr<ast::Value>>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Variant {
  pub name:   String,
  pub model:  NodeId,
  pub x:      Option<F64Eq>,
  pub y:      Option<F64Eq>,
  pub uvlock: Option<bool>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Model {
  pub path: Path,
}

pub fn parse_blockstate(
  db: &dyn HirDatabase,
  file_id: FileId,
) -> (Arc<Blockstate>, Arc<BlockstateSourceMap>, Arc<Diagnostics>) {
  let json = db.parse_json(file_id);

  let mut diagnostics = Diagnostics::new();
  let mut model = Blockstate::default();
  let mut source_map = BlockstateSourceMap::default();

  let tree = json.tree();

  parse::parse(&mut model, &mut source_map, &mut diagnostics, &tree);

  (Arc::new(model), Arc::new(source_map), Arc::new(diagnostics))
}

pub fn validate_blockstate(db: &dyn HirDatabase, file_id: FileId) -> Arc<Diagnostics> {
  // TODO: It might be nice to make this not dependent on the syntax tree
  // directly. Ideally, it'd only be dependent on `parse_model` and the model's
  // parent.
  let json = db.parse_json(file_id);

  let (_, source_map, diagnostics) = parse_blockstate(db, file_id);
  let mut diagnostics = (&*diagnostics).clone();

  validate::validate(db, file_id, &source_map, &json, &mut diagnostics);

  Arc::new(diagnostics)
}

pub fn ancestry(db: &dyn HirDatabase, file: FileId) -> Vec<FileId> {
  let mut ancestry = if let Some(ref parent) = db.parse_model(file).parent {
    if let Some(parent) = db.lookup_model(parent.clone()) {
      db.model_ancestry(parent)
    } else {
      vec![]
    }
  } else {
    vec![]
  };

  ancestry.push(file);

  ancestry
}
