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
        Node::TextureDef(texture_def) => self.validate_texture_def(id, &texture_def),
        Node::Texture(texture) => self.validate_texture(id, &texture),
        _ => {}
      }
    }
  }

  fn validate_texture_def(&mut self, id: NodeId, texture: &TextureDef) {
    let is_used = self.model.nodes.values().any(|node| match node {
      Node::Texture(Texture::Reference(name)) => texture.name == *name,
      _ => false,
    });

    // FIXME: Need to check if the texture is used in the parent model as well.
    if !is_used && false {
      self.diagnostics.warn(
        self.source_map.texture_defs[&id].to_node(&self.json),
        format!("texture `{}` is defined but not used", texture.name),
      );
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
