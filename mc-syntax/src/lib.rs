use std::marker::PhantomData;

pub mod ast;
mod node;
mod parse;

use ast::AstNode;
pub use ast::Json;
use mc_parser::SyntaxKind;
use node::SyntaxNode;
use rowan::{GreenNode, TextSize};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Parse<T> {
  green:  GreenNode,
  errors: Vec<SyntaxError>,
  _ty:    PhantomData<fn() -> T>,
}

impl Json {
  pub fn parse(text: &str) -> Parse<Json> {
    // TODO
    let (green, errors) = parse::parse_text(text);
    let root = SyntaxNode::new_root(green.clone());

    assert_eq!(root.kind(), SyntaxKind::JSON);
    Parse { green, errors, _ty: PhantomData }
  }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SyntaxError {
  pub message: String,
  pub offset:  TextSize,
}

impl SyntaxError {
  pub fn new_at_offset(message: String, offset: TextSize) -> Self { Self { message, offset } }
}

impl<T> Parse<T> {
  pub fn syntax_node(&self) -> SyntaxNode { SyntaxNode::new_root(self.green.clone()) }
  pub fn errors(&self) -> &[SyntaxError] { &self.errors }
}

impl<T: AstNode> Parse<T> {
  pub fn tree(&self) -> T { T::cast(self.syntax_node()).unwrap() }
}
