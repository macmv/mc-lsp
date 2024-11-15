//! A high-level parser for pulling things out of json, and producing
//! diagnostics.

use std::collections::HashSet;

use crate::diagnostic::Diagnostics;
use mc_syntax::ast::{self, AstNode};

pub struct Parser<'a> {
  pub diagnostics: &'a mut Diagnostics,
}

impl<'a> Parser<'a> {
  pub fn new(diagnostics: &'a mut Diagnostics) -> Self { Self { diagnostics } }

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

  pub fn string(&mut self, p: &ast::Value) -> Option<String> {
    match p.as_str() {
      Some(s) => Some(s),
      None => {
        self.diagnostics.error(p.syntax(), "expected string");
        None
      }
    }
  }
}
