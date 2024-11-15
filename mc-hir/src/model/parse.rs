use crate::{diagnostic::Diagnostics, parse::Parser};
use la_arena::RawIdx;
use mc_source::ModelPath;
use mc_syntax::{
  ast::{self, AstNode},
  AstPtr, Json,
};

use super::*;

struct ModelParser<'a> {
  parser:     Parser<'a>,
  model:      &'a mut Model,
  source_map: &'a mut ModelSourceMap,
}

pub fn parse(
  model: &mut Model,
  source_map: &mut ModelSourceMap,
  diagnostics: &mut Diagnostics,
  json: &Json,
) {
  let mut parser = ModelParser { parser: Parser::new(diagnostics), model, source_map };
  parser.parse_root(json);
}

impl ModelParser<'_> {
  fn parse_root(&mut self, json: &ast::Json) {
    let Some(root) = json.value() else { return };
    let Some(obj) = self.parser.object(root) else { return };
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
        "gui_light" => {}
        "display" => {}
        _ => self.parser.diagnostics.warn(key.syntax(), format!("unknown key `{key}`")),
      }
    }
  }

  fn parse_path(&mut self, p: ast::Value) -> Option<ModelPath> {
    let Some(path) = p.as_str() else {
      self.parser.diagnostics.error(p.syntax(), "expected string");
      return None;
    };
    Some(ModelPath { path: path.parse().ok()? })
  }

  fn parse_textures(&mut self, textures: ast::Value) {
    let Some(obj) = self.parser.object(textures) else { return };
    for element in obj.elements() {
      let Some(key) = element.key().map(|k| k.parse_text()) else { continue };
      let Some(value) = element.value() else { continue };
      let Some(texture) = self.parser.string(&value) else { continue };

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

    let obj = self.parser.object(e)?;
    for (key, value) in obj.iter() {
      match key.parse_text().as_str() {
        "from" => element.from = self.parse_pos(value),
        "to" => element.to = self.parse_pos(value),
        "faces" => element.faces = self.parse_faces(value),
        "rotation" => {}
        _ => self.parser.diagnostics.warn(key.syntax(), format!("unknown key `{key}`")),
      }
    }

    Some(self.alloc(obj, element))
  }

  fn parse_pos(&mut self, p: ast::Value) -> Pos {
    let mut pos = Pos::default();

    match p {
      ast::Value::Array(ref elems) if elems.values().count() == 3 => {
        for (i, elem) in elems.values().enumerate().take(3) {
          let Some(n) = self.parser.float(&elem) else { continue };

          match i {
            0 => pos.x = n.into(),
            1 => pos.y = n.into(),
            2 => pos.z = n.into(),
            _ => {}
          }
        }
      }

      ast::Value::Array(ref elems) => {
        self.parser.diagnostics.error(elems.syntax(), "expected 3 elements");
      }

      _ => {
        self.parser.diagnostics.error(p.syntax(), "expected array");
      }
    }

    pos
  }

  fn parse_faces(&mut self, f: ast::Value) -> Faces {
    let mut faces = Faces::default();

    let Some(obj) = self.parser.object(f) else { return faces };
    for (key, value) in obj.iter() {
      let Some(face) = self.parse_face(value) else { continue };

      match key.parse_text().as_str() {
        "north" => faces.north = Some(face),
        "east" => faces.east = Some(face),
        "south" => faces.south = Some(face),
        "west" => faces.west = Some(face),
        "up" => faces.up = Some(face),
        "down" => faces.down = Some(face),
        _ => self.parser.diagnostics.warn(key.syntax(), format!("unknown key `{key}`")),
      }
    }

    faces
  }

  fn parse_face(&mut self, f: ast::Value) -> Option<NodeId> {
    let mut face = Face {
      uv:      [0.0.into(), 0.0.into(), 16.0.into(), 16.0.into()],
      texture: NodeId::from_raw(RawIdx::from_u32(0)),
    };

    let obj = self.parser.object(f)?;

    for (key, value) in obj.iter() {
      match key.parse_text().as_str() {
        "uv" => {
          if let Some(arr) = self.parser.array(value) {
            for (i, item) in arr.values().enumerate() {
              face.uv[i] = self.parser.float(&item).unwrap_or_default().into();
            }
          }
        }
        "rotation" => {
          if let Some(n) = self.parser.int(&value) {
            if n % 45 != 0 {
              self.parser.diagnostics.error(value.syntax(), "rotation must be a multiple of 45");
            }
          }
        }
        "texture" => {
          let Some(texture) = value.as_str() else { continue };
          let Some(name) = texture.strip_prefix("#") else { continue };
          let node = self.alloc(value, Texture::Reference(name.to_string()));

          face.texture = node;
        }
        "cullface" => {}
        "tintindex" => {
          // TODO: Store this, and then tint the thing green.
          self.parser.int(&value);
        }
        _ => self.parser.diagnostics.warn(key.syntax(), format!("unknown key `{key}`")),
      }
    }

    Some(self.alloc(obj, face))
  }

  fn alloc<T: ModelNode>(&mut self, elem: T::Ast, node: T) -> NodeId { node.alloc(&elem, self) }
}

trait ModelNode {
  type Ast;

  fn alloc(self, elem: &Self::Ast, parser: &mut ModelParser) -> NodeId;
}

impl ModelNode for Parent {
  type Ast = ast::Value;

  fn alloc(self, elem: &Self::Ast, parser: &mut ModelParser) -> NodeId {
    let id = parser.model.nodes.alloc(Node::Parent(self));
    parser.source_map.parent.insert(id, AstPtr::new(&elem));
    parser.source_map.ast_values.insert(AstPtr::new(&elem), id);
    id
  }
}

impl ModelNode for TextureDef {
  type Ast = ast::Element;

  fn alloc(self, elem: &Self::Ast, parser: &mut ModelParser) -> NodeId {
    let id = parser.model.nodes.alloc(Node::TextureDef(self));
    parser.model.texture_defs.push(id);
    parser.source_map.texture_defs.insert(id, AstPtr::new(&elem));
    parser.source_map.ast_elements.insert(AstPtr::new(&elem), id);
    id
  }
}

impl ModelNode for Element {
  type Ast = ast::Object;

  fn alloc(self, elem: &Self::Ast, parser: &mut ModelParser) -> NodeId {
    let id = parser.model.nodes.alloc(Node::Element(self));
    parser.source_map.elements.insert(id, AstPtr::new(&elem));
    id
  }
}

impl ModelNode for Face {
  type Ast = ast::Object;

  fn alloc(self, elem: &Self::Ast, parser: &mut ModelParser) -> NodeId {
    let id = parser.model.nodes.alloc(Node::Face(self));
    parser.source_map.elements.insert(id, AstPtr::new(&elem));
    id
  }
}

impl ModelNode for Texture {
  type Ast = ast::Value;

  fn alloc(self, elem: &Self::Ast, parser: &mut ModelParser) -> NodeId {
    let id = parser.model.nodes.alloc(Node::Texture(self));
    parser.source_map.textures.insert(id, AstPtr::new(&elem));
    parser.source_map.ast_values.insert(AstPtr::new(&elem), id);
    id
  }
}
