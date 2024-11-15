use std::{collections::HashMap, sync::Arc};

mod parse;
mod validate;

use la_arena::{Arena, Idx};
use mc_source::{FileId, ModelPath};
use mc_syntax::{ast, AstPtr};

use crate::{diagnostic::Diagnostics, HirDatabase};

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Model {
  pub parent: Option<ModelPath>,

  pub nodes: Arena<Node>,

  pub texture_defs: Vec<NodeId>,
}

pub type NodeId = Idx<Node>;

#[derive(Debug, PartialEq, Eq)]
pub enum Node {
  Parent(Parent),
  TextureDef(TextureDef),
  Texture(Texture),
  Element(Element),
  Face(Face),
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct ModelSourceMap {
  pub ast_values:   HashMap<AstPtr<ast::Value>, NodeId>,
  pub ast_elements: HashMap<AstPtr<ast::Element>, NodeId>,

  pub parent:       HashMap<NodeId, AstPtr<ast::Value>>,
  pub texture_defs: HashMap<NodeId, AstPtr<ast::Element>>,
  pub textures:     HashMap<NodeId, AstPtr<ast::Value>>,
  pub elements:     HashMap<NodeId, AstPtr<ast::Object>>,
  pub faces:        HashMap<NodeId, AstPtr<ast::Object>>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Parent {
  pub path: ModelPath,
}

#[derive(Debug, PartialEq, Eq)]
pub struct TextureDef {
  pub name:  String,
  pub value: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Texture {
  Reference(String),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Element {
  pub from:  Pos,
  pub to:    Pos,
  pub faces: Faces,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Faces {
  pub north: Option<NodeId>,
  pub east:  Option<NodeId>,
  pub south: Option<NodeId>,
  pub west:  Option<NodeId>,
  pub up:    Option<NodeId>,
  pub down:  Option<NodeId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Face {
  pub uv:      [F64Eq; 4],
  pub texture: NodeId,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct F64Eq(pub f64);

impl From<f64> for F64Eq {
  fn from(f: f64) -> Self { Self(f) }
}
impl Into<f64> for F64Eq {
  fn into(self) -> f64 { self.0 }
}

impl Eq for F64Eq {}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Pos {
  pub x: F64Eq,
  pub y: F64Eq,
  pub z: F64Eq,
}

pub fn parse_model(
  db: &dyn HirDatabase,
  file_id: FileId,
) -> (Arc<Model>, Arc<ModelSourceMap>, Arc<Diagnostics>) {
  let json = db.parse_json(file_id);

  let mut diagnostics = Diagnostics::new();
  let mut model = Model::default();
  let mut source_map = ModelSourceMap::default();

  let tree = json.tree();

  parse::parse(&mut model, &mut source_map, &mut diagnostics, &tree);

  (Arc::new(model), Arc::new(source_map), Arc::new(diagnostics))
}

pub fn validate_model(db: &dyn HirDatabase, file_id: FileId) -> Arc<Diagnostics> {
  // TODO: It might be nice to make this not dependent on the syntax tree
  // directly. Ideally, it'd only be dependent on `parse_model` and the model's
  // parent.
  let json = db.parse_json(file_id);

  let (_, source_map, diagnostics) = parse_model(db, file_id);
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
