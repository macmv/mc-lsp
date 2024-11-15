use std::collections::HashSet;

use crate::diagnostic::Diagnostics;
use mc_source::{ModelPath, Path};
use mc_syntax::{
  ast::{self, AstNode},
  AstPtr, Json,
};

use super::*;

struct Parser<'a> {
  blockstate:  &'a mut Blockstate,
  source_map:  &'a mut BlockstateSourceMap,
  diagnostics: &'a mut Diagnostics,
}

pub fn parse(
  model: &mut Blockstate,
  source_map: &mut BlockstateSourceMap,
  diagnostics: &mut Diagnostics,
  json: &Json,
) {
  let mut parser = Parser { blockstate: model, source_map, diagnostics };
  parser.parse_root(json);
}

impl Parser<'_> {
  fn parse_root(&mut self, json: &ast::Json) {
    let Some(root) = json.value() else { return };
    let Some(obj) = self.parse_object(root) else { return };
    for (key, value) in obj.iter() {
      match key.parse_text().as_str() {
        "variants" => {
          let Some(variants) = self.parse_object(value) else { continue };
          for element in variants.elements() {
            let Some(key) = element.key().map(|k| k.parse_text()) else { continue };
            let Some(value) = element.value() else { continue };

            if let Some(variant) = self.parse_variant(key, value) {
              self.alloc(element, variant);
            }
          }
        }
        _ => self.diagnostics.warn(key.syntax(), format!("unknown key `{key}`")),
      }
    }
  }

  fn parse_variant(&mut self, key: String, e: ast::Value) -> Option<Variant> {
    let mut variant = Variant {
      name:   key,
      model:  ModelPath { path: Path::new() },
      x:      None,
      y:      None,
      uvlock: None,
    };

    let obj = self.parse_object(e)?;
    for (key, value) in obj.iter() {
      match key.parse_text().as_str() {
        "x" => variant.x = Some(F64Eq(self.float(&value)?)),
        "y" => variant.y = Some(F64Eq(self.float(&value)?)),
        "model" => variant.model = self.parse_path(value)?,
        _ => self.diagnostics.warn(key.syntax(), format!("unknown key `{key}`")),
      }
    }

    Some(variant)
  }

  fn parse_path(&mut self, p: ast::Value) -> Option<ModelPath> {
    let Some(path) = p.as_str() else {
      self.diagnostics.error(p.syntax(), "expected string");
      return None;
    };
    Some(ModelPath { path: path.parse().ok()? })
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

  fn float(&mut self, p: &ast::Value) -> Option<f64> {
    match p.as_f64() {
      Some(n) => Some(n),
      None => {
        self.diagnostics.error(p.syntax(), "expected float");
        None
      }
    }
  }

  fn alloc<T: BlockstateNode>(&mut self, elem: T::Ast, node: T) -> NodeId {
    node.alloc(&elem, self)
  }
}

trait BlockstateNode {
  type Ast;

  fn alloc(self, elem: &Self::Ast, parser: &mut Parser) -> NodeId;
}

impl BlockstateNode for Variant {
  type Ast = ast::Element;

  fn alloc(self, elem: &Self::Ast, parser: &mut Parser) -> NodeId {
    let id = parser.blockstate.nodes.alloc(Node::Variant(self));
    parser.source_map.variants.insert(id, AstPtr::new(&elem));
    parser.source_map.ast_variants.insert(AstPtr::new(&elem), id);
    id
  }
}
