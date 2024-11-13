use crate::diagnostic::Diagnostics;
use la_arena::RawIdx;
use mc_syntax::{
  ast::{self, AstNode},
  AstPtr, Json,
};

use super::*;

struct Parser<'a> {
  model:       &'a mut Model,
  source_map:  &'a mut ModelSourceMap,
  diagnostics: &'a mut Diagnostics,
}

pub fn parse(
  model: &mut Model,
  source_map: &mut ModelSourceMap,
  diagnostics: &mut Diagnostics,
  json: &Json,
) {
  let mut parser = Parser { model, source_map, diagnostics };
  parser.parse_root(json);
}

impl Parser<'_> {
  fn parse_root(&mut self, json: &ast::Json) {
    let Some(root) = json.value() else { return };
    let Some(obj) = self.parse_object(root) else { return };
    for (key, value) in obj.iter() {
      match key.parse_text().as_str() {
        "parent" => {
          if let Some(path) = self.parse_path(value.clone()) {
            self.alloc(value, Parent { path: path.clone() });
            self.model.parent = Some(path);
          }
        }
        "textures" => self.parse_textures(value),
        "elements" => self.parse_elements(value),
        _ => self.diagnostics.warn(key.syntax(), format!("unknown key `{key}`")),
      }
    }
  }

  fn parse_path(&mut self, p: ast::Value) -> Option<ModelPath> {
    let Some(path) = p.as_str() else {
      self.diagnostics.error(p.syntax(), "expected string");
      return None;
    };
    Some(ModelPath { path: path.parse().ok()? })
  }

  fn parse_textures(&mut self, textures: ast::Value) {
    let Some(obj) = self.parse_object(textures) else { return };
    for element in obj.elements() {
      let Some(key) = element.key().map(|k| k.parse_text()) else { continue };
      let Some(value) = element.value() else { continue };
      let Some(texture) = self.str(&value) else { continue };

      self.alloc(element, TextureDef { name: key, value: texture.to_string() });
    }
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

    let obj = self.parse_object(e)?;
    for (key, value) in obj.iter() {
      match key.parse_text().as_str() {
        "from" => element.from = self.parse_pos(value),
        "to" => element.to = self.parse_pos(value),
        "faces" => element.faces = self.parse_faces(value),
        "rotation" => {}
        _ => self.diagnostics.warn(key.syntax(), format!("unknown key `{key}`")),
      }
    }

    Some(self.alloc(obj, element))
  }

  fn parse_pos(&mut self, p: ast::Value) -> Pos {
    let mut pos = Pos::default();

    match p {
      ast::Value::Array(ref elems) if elems.values().count() == 3 => {
        for (i, elem) in elems.values().enumerate().take(3) {
          let Some(n) = elem.as_i64() else {
            self.diagnostics.error(elem.syntax(), "expected number");
            continue;
          };

          match i {
            0 => pos.x = n,
            1 => pos.y = n,
            2 => pos.z = n,
            _ => {}
          }
        }
      }

      ast::Value::Array(ref elems) => {
        self.diagnostics.error(elems.syntax(), "expected 3 elements");
      }

      _ => {
        self.diagnostics.error(p.syntax(), "expected array");
      }
    }

    pos
  }

  fn parse_faces(&mut self, f: ast::Value) -> Faces {
    let mut faces = Faces::default();

    let Some(obj) = self.parse_object(f) else { return faces };
    for (key, value) in obj.iter() {
      let Some(face) = self.parse_face(value) else { continue };

      match key.parse_text().as_str() {
        "north" => faces.north = Some(face),
        "east" => faces.east = Some(face),
        "south" => faces.south = Some(face),
        "west" => faces.west = Some(face),
        "up" => faces.up = Some(face),
        "down" => faces.down = Some(face),
        _ => self.diagnostics.warn(key.syntax(), format!("unknown key `{key}`")),
      }
    }

    faces
  }

  fn parse_face(&mut self, f: ast::Value) -> Option<NodeId> {
    let mut face =
      Face { uv: [0; 4], texture: NodeId::from_raw(RawIdx::from_u32(0)), cull: false };

    let obj = self.parse_object(f)?;

    for (key, value) in obj.iter() {
      match key.parse_text().as_str() {
        "uv" => {
          if let Some(arr) = self.arr(value) {
            for (i, item) in arr.values().enumerate() {
              face.uv[i] = item.as_i64().unwrap_or(0);
            }
          }
        }
        "rotation" => {
          if let Some(n) = self.int(&value) {
            if n % 45 != 0 {
              self.diagnostics.error(value.syntax(), "rotation must be a multiple of 45");
            }
          }
        }
        "texture" => {
          let Some(texture) = value.as_str() else { continue };
          let Some(name) = texture.strip_prefix("#") else { continue };
          let node = self.alloc(value, Texture::Reference(name.to_string()));

          face.texture = node;
        }
        "cull" => face.cull = value.as_bool().unwrap_or(false),
        _ => self.diagnostics.warn(key.syntax(), format!("unknown key `{key}`")),
      }
    }

    Some(self.alloc(obj, face))
  }

  fn parse_object(&mut self, object: ast::Value) -> Option<ast::Object> {
    match object {
      ast::Value::Object(obj) => {
        let mut keys = HashSet::new();

        for elem in obj.elements() {
          let Some(key) = elem.key() else { continue };
          let key_str = key.parse_text();
          if !keys.insert(key_str.clone()) {
            self.diagnostics.error(key.syntax(), "duplicate key");
          }
        }

        Some(obj)
      }
      _ => {
        self.diagnostics.error(object.syntax(), "expected object");
        None
      }
    }
  }

  fn arr(&mut self, p: ast::Value) -> Option<ast::Array> {
    match p {
      ast::Value::Array(arr) => Some(arr),
      _ => {
        self.diagnostics.error(p.syntax(), "expected array");
        None
      }
    }
  }
  fn int(&mut self, p: &ast::Value) -> Option<i64> {
    match p.as_i64() {
      Some(n) => Some(n),
      None => {
        self.diagnostics.error(p.syntax(), "expected number");
        None
      }
    }
  }
  fn str(&mut self, p: &ast::Value) -> Option<String> {
    match p.as_str() {
      Some(s) => Some(s),
      None => {
        self.diagnostics.error(p.syntax(), "expected string");
        None
      }
    }
  }

  fn alloc<T: ModelNode>(&mut self, elem: T::Ast, node: T) -> NodeId { node.alloc(&elem, self) }
}

trait ModelNode {
  type Ast;

  fn alloc(self, elem: &Self::Ast, parser: &mut Parser) -> NodeId;
}

impl ModelNode for Parent {
  type Ast = ast::Value;

  fn alloc(self, elem: &Self::Ast, parser: &mut Parser) -> NodeId {
    let id = parser.model.nodes.alloc(Node::Parent(self));
    parser.source_map.parent.insert(id, AstPtr::new(&elem));
    parser.source_map.ast_values.insert(AstPtr::new(&elem), id);
    id
  }
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
