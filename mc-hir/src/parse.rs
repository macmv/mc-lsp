//! A high-level parser for pulling things out of json, and producing
//! diagnostics.

use std::collections::HashSet;

use crate::diagnostic::Diagnostics;
use mc_source::{TextRange, TextSize};
use mc_syntax::{
  ast::{self, AstNode},
  Json,
};

pub struct Parser<'a> {
  pub json:        &'a Json,
  pub diagnostics: &'a mut Diagnostics,
}

impl<'a> Parser<'a> {
  pub fn new(json: &'a Json, diagnostics: &'a mut Diagnostics) -> Self {
    Parser { json, diagnostics }
  }

  pub fn object(&mut self, object: ast::Value) -> Option<ast::Object> {
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

  pub fn array(&mut self, p: ast::Value) -> Option<ast::Array> {
    match p {
      ast::Value::Array(arr) => Some(arr),
      _ => {
        self.diagnostics.error(p.syntax(), "expected array");
        None
      }
    }
  }

  pub fn float(&mut self, p: &ast::Value) -> Option<f64> {
    match p.as_f64() {
      Some(n) => Some(n),
      None => {
        self.diagnostics.error(p.syntax(), "expected float");
        None
      }
    }
  }

  pub fn int(&mut self, p: &ast::Value) -> Option<i64> {
    match p.as_i64() {
      Some(n) => Some(n),
      None => {
        self.diagnostics.error(p.syntax(), "expected integer");
        None
      }
    }
  }

  pub fn bool(&mut self, p: &ast::Value) -> Option<bool> {
    match p.as_bool() {
      Some(n) => Some(n),
      None => {
        self.diagnostics.error(p.syntax(), "expected boolean");
        None
      }
    }
  }

  pub fn string(&mut self, p: &ast::Value) -> Option<String> {
    match p.as_str() {
      Some(s) => Some(s),
      None => {
        self.diagnostics.error(p.syntax(), "expected string");
        None
      }
    }
  }

  pub fn warn_unknown_key(&mut self, key: ast::Key) {
    let element = ast::Element::cast(key.syntax().parent().unwrap()).unwrap();

    let remove_range = element.syntax().text_range();
    let mut i = u32::from(remove_range.end()) as usize;
    let text = self.json.syntax().text().to_string();
    while let Some(c) = text.as_bytes().get(i) {
      match c {
        // Eat whitespace. This works surprisingly well: we're removing an element, so we want to
        // remove everything from the start of the current element (which is indented), up to the
        // next element's start (which is also indented). So, the end result here is the current
        // element is seemlessly removed.
        b' ' | b'\t' | b'\n' | b'\r' => i += 1,
        _ => break,
      }
    }
    let remove_range = TextRange::new(remove_range.start(), TextSize::from(i as u32));

    self
      .diagnostics
      .warn(key.syntax(), format!("unknown key `{key}`"))
      .suggest_remove("remove the unknown key", remove_range);
  }
}
