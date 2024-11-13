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
    self.parse_object(root, |p, _, key_syntax, key, value| match key {
      "parent" => p.model.parent = p.parse_path(value),
      "textures" => p.parse_textures(value),
      "elements" => p.parse_elements(value),
      _ => p.diagnostics.warn(key_syntax.syntax(), format!("unknown key `{key}`")),
    });
  }

  fn parse_path(&mut self, p: ast::Value) -> Option<ModelPath> {
    let path = p.as_str()?;
    Some(ModelPath(path.parse().ok()?))
  }

  fn parse_textures(&mut self, textures: ast::Value) {
    self.parse_object(textures, |p, elem, _, key, value| {
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

    let obj = self.parse_object(e, |p, _, key_syntax, key, value| match key {
      "from" => element.from = p.parse_pos(value),
      "to" => element.to = p.parse_pos(value),
      "faces" => element.faces = p.parse_faces(value),
      "rotation" => {}
      _ => p.diagnostics.warn(key_syntax.syntax(), format!("unknown key `{key}`")),
    });

    obj.map(|o| self.alloc(o, element))
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

    self.parse_object(f, |p, _, key_syntax, key, value| {
      let Some(face) = p.parse_face(value) else { return };

      match key {
        "north" => faces.north = Some(face),
        "east" => faces.east = Some(face),
        "south" => faces.south = Some(face),
        "west" => faces.west = Some(face),
        "up" => faces.up = Some(face),
        "down" => faces.down = Some(face),
        _ => p.diagnostics.warn(key_syntax.syntax(), format!("unknown key `{key}`")),
      }
    });

    faces
  }

  fn parse_face(&mut self, f: ast::Value) -> Option<NodeId> {
    let mut face =
      Face { uv: [0; 4], texture: NodeId::from_raw(RawIdx::from_u32(0)), cull: false };

    let obj = self.parse_object(f, |p, _, key_syntax, key, value| match key {
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
      _ => p.diagnostics.warn(key_syntax.syntax(), format!("unknown key `{key}`")),
    });

    obj.map(|o| self.alloc(o, face))
  }

  fn parse_object(
    &mut self,
    object: ast::Value,
    mut f: impl FnMut(&mut Parser, ast::Element, ast::Key, &str, ast::Value),
  ) -> Option<ast::Object> {
    match object {
      ast::Value::Object(obj) => {
        let mut keys = HashSet::new();

        for elem in obj.elements() {
          let Some(key) = elem.key() else { continue };
          let key_str = key.parse_text();
          if !keys.insert(key_str.clone()) {
            self.diagnostics.error(key.syntax(), "duplicate key");
            continue;
          }
          let Some(value) = elem.value() else { continue };

          f(self, elem, key, &key_str, value);
        }

        Some(obj)
      }
      _ => {
        self.diagnostics.error(object.syntax(), "expected object");
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
