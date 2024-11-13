use std::{
  collections::{HashMap, HashSet},
  sync::Arc,
};

mod parse;
mod validate;

use la_arena::{Arena, Idx};
use mc_source::FileId;
use mc_syntax::{ast, AstPtr};

use crate::{diagnostic::Diagnostics, HirDatabase, Path};

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Model {
  pub parent: Option<ModelPath>,

  pub nodes: Arena<Node>,

  pub texture_defs: Vec<NodeId>,
}

pub type NodeId = Idx<Node>;

#[derive(Debug, PartialEq, Eq)]
pub enum Node {
  TextureDef(TextureDef),
  Texture(Texture),
  Element(Element),
  Face(Face),
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct ModelSourceMap {
  pub ast_values:   HashMap<AstPtr<ast::Value>, NodeId>,
  pub ast_elements: HashMap<AstPtr<ast::Element>, NodeId>,

  pub texture_defs: HashMap<NodeId, AstPtr<ast::Element>>,
  pub textures:     HashMap<NodeId, AstPtr<ast::Value>>,
  pub elements:     HashMap<NodeId, AstPtr<ast::Object>>,
  pub faces:        HashMap<NodeId, AstPtr<ast::Object>>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ModelPath(Path);

#[derive(Debug, PartialEq, Eq)]
pub struct TextureDef {
  pub name:  String,
  pub value: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Texture {
  Reference(String),
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Element {
  pub from:  Pos,
  pub to:    Pos,
  pub faces: Faces,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Faces {
  pub north: Option<NodeId>,
  pub east:  Option<NodeId>,
  pub south: Option<NodeId>,
  pub west:  Option<NodeId>,
  pub up:    Option<NodeId>,
  pub down:  Option<NodeId>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Face {
  pub uv:      [i64; 4], // FIXME: `f64` but Eq
  pub texture: NodeId,
  pub cull:    bool,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Pos {
  pub x: i64,
  pub y: i64,
  pub z: i64,
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
  validate::validate(&model, &source_map, &json, &mut diagnostics);

  (Arc::new(model), Arc::new(source_map), Arc::new(diagnostics))
}