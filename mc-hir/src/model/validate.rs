use ast::Json;
use mc_syntax::Parse;

use crate::diagnostic::Diagnostics;

use super::*;

struct Validator<'a> {
  model:       &'a Model,
  source_map:  &'a ModelSourceMap,
  json:        &'a Parse<Json>,
  diagnostics: &'a mut Diagnostics,
}

pub fn validate(
  model: &Model,
  source_map: &ModelSourceMap,
  json: &Parse<Json>,
  diagnostics: &mut Diagnostics,
) {
  let mut validator = Validator { model, source_map, json, diagnostics };
  validator.validate_model();
}

impl Validator<'_> {
  fn validate_model(&mut self) {
    for (id, node) in self.model.nodes.iter() {
      match node {
        Node::Texture(texture) => self.validate_texture(id, &texture),
        _ => {}
      }
    }
  }

  fn validate_texture(&mut self, id: NodeId, texture: &Texture) {
    match texture {
      Texture::Reference(name) => {
        if !self.model.texture_defs.iter().any(|id| {
          let texture_def = match self.model.nodes[*id] {
            Node::TextureDef(ref texture_def) => texture_def,
            _ => unreachable!(),
          };

          texture_def.name == *name
        }) {
          self.diagnostics.error(
            self.source_map.textures[&id].to_node(&self.json),
            format!("texture `{}` not found", name),
          );
        }
      }
    }
  }
}
