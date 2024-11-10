use std::sync::Arc;

use la_arena::{Arena, Idx};
use mc_source::FileId;
use mc_syntax::ast;

use crate::HirDatabase;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Model {
  pub nodes: Arena<Node>,

  pub textures: Vec<NodeId>,
}

pub type NodeId = Idx<Node>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
  TextureDef(TextureDef),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextureDef {
  pub name:  String,
  pub value: String,
}

struct Parser<'a> {
  model: &'a mut Model,
}

pub fn parse_model(db: &dyn HirDatabase, file_id: FileId) -> Arc<Model> {
  let json = db.parse_json(file_id);

  let mut model = Model { nodes: Arena::new(), textures: Vec::new() };
  let mut parser = Parser { model: &mut model };

  parser.parse_root(&json.tree());

  Arc::new(model)
}

impl Parser<'_> {
  fn parse_root(&mut self, json: &ast::Json) {
    let Some(root) = json.value() else { return };
    self.parse_object(root, |p, key, value| match key {
      "textures" => {
        p.parse_textures(value);
      }
      _ => {}
    });
  }

  fn parse_textures(&mut self, textures: ast::Value) {
    self.parse_object(textures, |p, key, value| {
      let Some(texture) = value.as_str() else { return };

      p.alloc_texture_def(TextureDef { name: key.to_string(), value: texture.to_string() });
    });
  }

  fn parse_object(&mut self, object: ast::Value, mut f: impl FnMut(&mut Parser, &str, ast::Value)) {
    match object {
      ast::Value::Object(obj) => {
        for elem in obj.elements() {
          let Some(key) = elem.key() else { continue };
          let key_str = key.parse_text();
          let Some(value) = elem.value() else { continue };

          f(self, &key_str, value);
        }
      }
      _ => {}
    }
  }

  fn alloc_texture_def(&mut self, texture_def: TextureDef) -> NodeId {
    let id = self.model.nodes.alloc(Node::TextureDef(texture_def));
    self.model.textures.push(id);
    id
  }
}
