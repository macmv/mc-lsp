use std::{
  collections::{HashMap, HashSet},
  sync::Arc,
};

use mc_source::{FileId, TextRange, TextSize};
use mc_syntax::{Json, Parse, SyntaxKind, SyntaxNode};

use crate::{diagnostic::Diagnostics, HirDatabase};

use super::{Blockstate, BlockstateSourceMap, Node};

struct ModelValidator<'a> {
  db:         &'a dyn HirDatabase,
  blockstate: Arc<Blockstate>,
}

struct Validator<'a> {
  db:      &'a dyn HirDatabase,
  model:   &'a Blockstate,
  file_id: FileId,

  source_map:  &'a BlockstateSourceMap,
  json:        &'a Parse<Json>,
  diagnostics: &'a mut Diagnostics,
}

pub fn validate(
  db: &dyn HirDatabase,
  file_id: FileId,
  source_map: &BlockstateSourceMap,
  json: &Parse<Json>,
  diagnostics: &mut Diagnostics,
) {
  let blockstate = db.parse_blockstate(file_id);
  let mut validator = Validator { db, model: &blockstate, file_id, source_map, json, diagnostics };
  validator.validate_blockstate();
}

struct Prop {
  key:   String,
  value: String,
}

impl Validator<'_> {
  fn validate_blockstate(&mut self) {
    for (id, node) in self.model.nodes.iter() {
      match node {
        Node::Variant(variant) => {
          self.check_prop_list(&variant.name, self.source_map.variants[&id].to_node(&self.json))
        }
        _ => {}
      }
    }

    // This is the inferred property map of this blockstate.
    let mut all_props = HashMap::<String, HashSet<String>>::new();

    for node in self.model.nodes.values() {
      match node {
        Node::Variant(variant) => {
          let props = self.parse_prop_list(&variant.name);
          for prop in props {
            all_props.entry(prop.key.clone()).or_default().insert(prop.value.clone());
          }
        }
        _ => {}
      }
    }

    // TODO: Check if all combinations have been specified.
  }

  fn check_prop_list(&mut self, s: &str, syntax: SyntaxNode) {
    // Special case: no properties.
    if s == "normal" {
      return;
    }

    if s == "" {
      self
        .diagnostics
        .error(syntax, "empty property list is not allowed")
        .hint("use 'normal' instead");
      return;
    }

    let mut i = 1; // Start at 1 to chop off the leading `"`.
    let mut prev_key = "";
    let mut seen = HashSet::new();

    for prop in s.split(',') {
      // FIXME: Need to handle escapes.
      let span = TextRange::new(
        syntax.text_range().start() + TextSize::from(i),
        syntax.text_range().start() + TextSize::from(i + prop.len() as u32),
      );

      if !prop.contains('=') {
        self
          .diagnostics
          .error(span, format!("invalid property `{}`", prop))
          .hint("properties should be in the form `key=value`");

        i += prop.len() as u32 + 1;
        continue;
      }

      let key = prop.split('=').next().unwrap();
      if key.is_empty() {
        self.diagnostics.error(span, format!("invalid empty property key`"));
      }

      if key < prev_key {
        self.diagnostics.error(span, format!("property keys must be in alphabetical order"));
      }
      prev_key = key;

      if !seen.insert(key) {
        self.diagnostics.error(span, format!("duplicate property key `{}`", key));
      }

      if !key.chars().all(|c| matches!(c, 'a'..='z' | '_')) {
        self
          .diagnostics
          .error(span, format!("invalid property key `{}`", key))
          .hint("property keys may only contain lowercase letters");
      }

      let value = prop.split('=').nth(1).unwrap();
      if value.is_empty() {
        self.diagnostics.error(span, format!("invalid empty property value"));
      }

      if !value.chars().all(|c| matches!(c, 'a'..='z' | '0'..='9' | '_')) {
        self
          .diagnostics
          .error(span, format!("invalid property value `{}`", value))
          .hint("property values may only contain lowercase letters or numbers");
      }

      i += prop.len() as u32 + 1;
    }
  }

  /// Parses the property list. Ignores any invalid properties.
  fn parse_prop_list(&self, s: &str) -> Vec<Prop> {
    let mut props = vec![];
    for prop in s.split(',') {
      let mut parts = prop.split('=');
      let key = parts.next().unwrap();
      let value = parts.next().unwrap_or("");
      props.push(Prop { key: key.to_string(), value: value.to_string() });
    }
    props
  }
}