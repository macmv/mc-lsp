use ast::Json;
use mc_source::ResolvedPath;
use mc_syntax::Parse;

use crate::diagnostic::Diagnostics;

use super::*;

struct ModelValidator<'a> {
  db:    &'a dyn HirDatabase,
  model: Arc<Model>,
}

struct Validator<'a> {
  db:      &'a dyn HirDatabase,
  model:   &'a Model,
  file_id: FileId,

  source_map:  &'a ModelSourceMap,
  json:        &'a Parse<Json>,
  diagnostics: &'a mut Diagnostics,
}

pub fn validate(
  db: &dyn HirDatabase,
  file_id: FileId,
  source_map: &ModelSourceMap,
  json: &Parse<Json>,
  diagnostics: &mut Diagnostics,
) {
  let model = db.parse_model(file_id);
  let mut validator = Validator { db, model: &model, file_id, source_map, json, diagnostics };
  validator.validate_model();
}

impl Validator<'_> {
  fn model_validator(&self) -> ModelValidator {
    ModelValidator { db: self.db, model: self.db.parse_model(self.file_id) }
  }

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
    let is_used = self.model_validator().is_texture_def_used(&texture.name);

    if !is_used {
      self.diagnostics.warn(
        self.source_map.texture_defs[&id].to_node(&self.json),
        format!("texture `{}` is defined but not used", texture.name),
      );
    }

    if !texture.value.starts_with("#") {
      let search_path =
        ResolvedPath::Texture(mc_source::TexturePath::new(texture.value.parse().unwrap()));

      // FIXME: Extract to a function.
      let workspace = self.db.workspace();
      let file = workspace.namespaces.iter().find_map(|n| {
        n.files.iter().find_map(|f| {
          if f.path().as_ref() == Some(&search_path) {
            Some(f.id)
          } else {
            None
          }
        })
      });

      if file.is_none() {
        self.diagnostics.error(
          self.source_map.texture_defs[&id].to_node(&self.json),
          format!("texture `{}` not found", texture.value),
        );
      }
    }
  }

  fn validate_texture(&mut self, id: NodeId, texture: &Texture) {
    // FIXME: We only want to run this check on the leaf models, that are used by
    // blockstates directly (as texture defs may exist in child models, which
    // are then picked up by the parent model).
    if false {
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
}

impl<'a> ModelValidator<'a> {
  fn parent(&self) -> Option<ModelValidator<'a>> {
    let parent = self.model.parent.clone()?;

    Some(ModelValidator {
      db:    self.db,
      model: self.db.parse_model(self.db.lookup_model(parent)?),
    })
  }

  fn is_texture_def_used(&self, name: &str) -> bool {
    if name == "particle" {
      return true;
    }

    self.model.nodes.values().any(|node| match node {
      Node::Texture(Texture::Reference(n)) => name == *n,
      Node::TextureDef(def) => def.value.strip_prefix("#") == Some(name),
      _ => false,
    }) || self.parent().is_some_and(|p| p.is_texture_def_used(name))
  }
}
