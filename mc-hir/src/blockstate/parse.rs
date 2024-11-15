use crate::{diagnostic::Diagnostics, parse::Parser};
use mc_source::{ModelPath, Path};
use mc_syntax::{
  ast::{self, AstNode},
  AstPtr, Json,
};

use super::*;

struct BlockstateParser<'a> {
  parser:     Parser<'a>,
  blockstate: &'a mut Blockstate,
  source_map: &'a mut BlockstateSourceMap,
}

pub fn parse(
  blockstate: &mut Blockstate,
  source_map: &mut BlockstateSourceMap,
  diagnostics: &mut Diagnostics,
  json: &Json,
) {
  let mut parser = BlockstateParser { parser: Parser::new(diagnostics), blockstate, source_map };
  parser.parse_root(json);
}

impl BlockstateParser<'_> {
  fn parse_root(&mut self, json: &ast::Json) {
    let Some(root) = json.value() else { return };
    let Some(obj) = self.parser.object(root) else { return };
    for (key, value) in obj.iter() {
      match key.parse_text().as_str() {
        "variants" => {
          let Some(variants) = self.parser.object(value) else { continue };
          for element in variants.elements() {
            let Some(key) = element.key().map(|k| k.parse_text()) else { continue };
            let Some(value) = element.value() else { continue };

            if let Some(variant) = self.parse_variant(key, value) {
              self.alloc(element, variant);
            }
          }
        }
        _ => {
          self.parser.diagnostics.warn(key.syntax(), format!("unknown key `{key}`"));
        }
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

    let obj = self.parser.object(e)?;
    for (key, value) in obj.iter() {
      match key.parse_text().as_str() {
        "model" => variant.model = self.parse_path(value)?,
        "x" => variant.x = Some(F64Eq(self.parser.float(&value)?)),
        "y" => variant.y = Some(F64Eq(self.parser.float(&value)?)),
        "uvlock" => variant.uvlock = Some(self.parser.bool(&value)?),
        _ => {
          self.parser.diagnostics.warn(key.syntax(), format!("unknown key `{key}`"));
        }
      }
    }

    Some(variant)
  }

  fn parse_path(&mut self, p: ast::Value) -> Option<ModelPath> {
    let Some(path) = p.as_str() else {
      self.parser.diagnostics.error(p.syntax(), "expected string");
      return None;
    };
    Some(ModelPath { path: path.parse().ok()? })
  }

  fn alloc<T: BlockstateNode>(&mut self, elem: T::Ast, node: T) -> NodeId {
    node.alloc(&elem, self)
  }
}

trait BlockstateNode {
  type Ast;

  fn alloc(self, elem: &Self::Ast, parser: &mut BlockstateParser) -> NodeId;
}

impl BlockstateNode for Variant {
  type Ast = ast::Element;

  fn alloc(self, elem: &Self::Ast, parser: &mut BlockstateParser) -> NodeId {
    let id = parser.blockstate.nodes.alloc(Node::Variant(self));
    parser.source_map.variants.insert(id, AstPtr::new(&elem));
    parser.source_map.ast_variants.insert(AstPtr::new(&elem), id);
    id
  }
}
