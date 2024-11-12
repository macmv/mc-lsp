use std::{
  collections::{HashMap, HashSet},
  sync::Arc,
};

use la_arena::{Arena, Idx, RawIdx};
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

struct Parser<'a> {
  model:       &'a mut Model,
  source_map:  &'a mut ModelSourceMap,
  diagnostics: &'a mut Diagnostics,
}

pub fn parse_model(
  db: &dyn HirDatabase,
  file_id: FileId,
) -> (Arc<Model>, Arc<ModelSourceMap>, Arc<Diagnostics>) {
  let json = db.parse_json(file_id);

  let mut diagnostics = Diagnostics::new();
  let mut model = Model::default();
  let mut source_map = ModelSourceMap::default();
  let mut parser =
    Parser { model: &mut model, source_map: &mut source_map, diagnostics: &mut diagnostics };

  parser.parse_root(&json.tree());

  (Arc::new(model), Arc::new(source_map), Arc::new(diagnostics))
}

impl Parser<'_> {
  fn parse_root(&mut self, json: &ast::Json) {
    let Some(root) = json.value() else { return };
    self.parse_object(root, |p, _, key, value| match key {
      "parent" => p.model.parent = p.parse_path(value),
      "textures" => p.parse_textures(value),
      "elements" => p.parse_elements(value),
      _ => {}
    });
  }

  fn parse_path(&mut self, p: ast::Value) -> Option<ModelPath> {
    let path = p.as_str()?;
    Some(ModelPath(path.parse().ok()?))
  }

  fn parse_textures(&mut self, textures: ast::Value) {
    self.parse_object(textures, |p, elem, key, value| {
      let Some(texture) = value.as_str() else { return };

      p.alloc(elem, TextureDef { name: key.to_string(), value: texture.to_string() });
    });
  }

  fn parse_elements(&mut self, elements: ast::Value) {
    if let ast::Value::Array(arr) = elements {
      for item in arr.values() {
        self.parse_element(item);
      }
    }
  }

  fn parse_element(&mut self, e: ast::Value) -> Option<NodeId> {
    let mut element = Element::default();

    let obj = self.parse_object(e, |p, _, key, value| match key {
      "from" => element.from = p.parse_pos(value),
      "to" => element.to = p.parse_pos(value),
      "faces" => element.faces = p.parse_faces(value),
      _ => {}
    });

    obj.map(|o| self.alloc(o, element))
  }

  fn parse_pos(&mut self, p: ast::Value) -> Pos {
    let mut pos = Pos::default();

    self.parse_object(p, |_, _, key, value| match key {
      "x" => pos.x = value.as_i64().unwrap_or(0),
      "y" => pos.y = value.as_i64().unwrap_or(0),
      "z" => pos.z = value.as_i64().unwrap_or(0),
      _ => {}
    });

    pos
  }

  fn parse_faces(&mut self, f: ast::Value) -> Faces {
    let mut faces = Faces::default();

    self.parse_object(f, |p, _, key, value| {
      let Some(face) = p.parse_face(value) else { return };

      match key {
        "north" => faces.north = Some(face),
        "east" => faces.east = Some(face),
        "south" => faces.south = Some(face),
        "west" => faces.west = Some(face),
        "up" => faces.up = Some(face),
        "down" => faces.down = Some(face),
        _ => {}
      }
    });

    faces
  }

  fn parse_face(&mut self, f: ast::Value) -> Option<NodeId> {
    let mut face =
      Face { uv: [0; 4], texture: NodeId::from_raw(RawIdx::from_u32(0)), cull: false };

    let obj = self.parse_object(f, |p, _, key, value| match key {
      "uv" => {
        if let ast::Value::Array(arr) = value {
          for (i, item) in arr.values().enumerate() {
            face.uv[i] = item.as_i64().unwrap_or(0);
          }
        }
      }
      "texture" => {
        let Some(texture) = value.as_str() else { return };
        let Some(name) = texture.strip_prefix("#") else { return };
        let node = p.alloc(value, Texture::Reference(name.to_string()));

        face.texture = node;
      }
      "cull" => face.cull = value.as_bool().unwrap_or(false),
      _ => {}
    });

    obj.map(|o| self.alloc(o, face))
  }

  fn parse_object(
    &mut self,
    object: ast::Value,
    mut f: impl FnMut(&mut Parser, ast::Element, &str, ast::Value),
  ) -> Option<ast::Object> {
    match object {
      ast::Value::Object(obj) => {
        let mut keys = HashSet::new();

        for elem in obj.elements() {
          let Some(key) = elem.key() else { continue };
          let key_str = key.parse_text();
          if !keys.insert(key_str.clone()) {
            self.diagnostics.error(key, "duplicate key");
            continue;
          }
          let Some(value) = elem.value() else { continue };

          f(self, elem, &key_str, value);
        }

        Some(obj)
      }
      _ => None,
    }
  }

  fn alloc<T: ModelNode>(&mut self, elem: T::Ast, node: T) -> NodeId { node.alloc(&elem, self) }
}

trait ModelNode {
  type Ast;

  fn alloc(self, elem: &Self::Ast, parser: &mut Parser) -> NodeId;
}

impl ModelNode for TextureDef {
  type Ast = ast::Element;

  fn alloc(self, elem: &Self::Ast, parser: &mut Parser) -> NodeId {
    let id = parser.model.nodes.alloc(Node::TextureDef(self));
    parser.model.texture_defs.push(id);
    parser.source_map.texture_defs.insert(id, AstPtr::new(&elem));
    parser.source_map.ast_elements.insert(AstPtr::new(&elem), id);
    id
  }
}

impl ModelNode for Element {
  type Ast = ast::Object;

  fn alloc(self, elem: &Self::Ast, parser: &mut Parser) -> NodeId {
    let id = parser.model.nodes.alloc(Node::Element(self));
    parser.source_map.elements.insert(id, AstPtr::new(&elem));
    id
  }
}

impl ModelNode for Face {
  type Ast = ast::Object;

  fn alloc(self, elem: &Self::Ast, parser: &mut Parser) -> NodeId {
    let id = parser.model.nodes.alloc(Node::Face(self));
    parser.source_map.elements.insert(id, AstPtr::new(&elem));
    id
  }
}

impl ModelNode for Texture {
  type Ast = ast::Value;

  fn alloc(self, elem: &Self::Ast, parser: &mut Parser) -> NodeId {
    let id = parser.model.nodes.alloc(Node::Texture(self));
    parser.source_map.textures.insert(id, AstPtr::new(&elem));
    parser.source_map.ast_values.insert(AstPtr::new(&elem), id);
    id
  }
}
