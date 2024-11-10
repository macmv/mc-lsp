use std::sync::Arc;

use la_arena::{Arena, Idx};
use mc_source::FileId;
use mc_syntax::ast;

use crate::HirDatabase;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Model {
  pub nodes: Arena<Node>,
}

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
  nodes: &'a mut Arena<Node>,
}

pub fn parse_model(db: &dyn HirDatabase, file_id: FileId) -> Arc<Model> {
  let json = db.parse_json(file_id);

  let mut nodes = Arena::new();
  let mut parser = Parser { nodes: &mut nodes };

  parser.parse_root(&json.tree());

  Arc::new(Model { nodes })
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

      p.nodes
        .alloc(Node::TextureDef(TextureDef { name: key.to_string(), value: texture.to_string() }));
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
}
