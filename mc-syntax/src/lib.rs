use std::{
  hash::{Hash, Hasher},
  marker::PhantomData,
};

pub mod ast;
mod node;
mod parse;

use ast::AstNode;
pub use ast::Json;
use node::Mc;
use rowan::{GreenNode, TextSize};

pub use mc_parser::{SyntaxKind, T};
pub use node::{SyntaxNode, SyntaxToken};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Parse<T> {
  green:  GreenNode,
  errors: Vec<SyntaxError>,
  _ty:    PhantomData<fn() -> T>,
}

pub type SyntaxNodePtr = rowan::ast::SyntaxNodePtr<Mc>;

#[derive(Debug, PartialEq, Eq)]
pub struct AstPtr<T> {
  ptr:      SyntaxNodePtr,
  _phantom: std::marker::PhantomData<fn() -> T>,
}

impl<T> Clone for AstPtr<T> {
  fn clone(&self) -> Self { AstPtr { ptr: self.ptr, _phantom: PhantomData } }
}
impl<T> Copy for AstPtr<T> {}

impl<T> Hash for AstPtr<T> {
  fn hash<H: Hasher>(&self, state: &mut H) { self.ptr.hash(state) }
}

impl<T: AstNode> AstPtr<T> {
  pub fn new(node: &T) -> Self {
    AstPtr { ptr: SyntaxNodePtr::new(node.syntax()), _phantom: PhantomData }
  }

  pub fn to_node(&self, root: &Parse<Json>) -> SyntaxNode { self.ptr.to_node(&root.syntax_node()) }

  pub fn tree(&self, root: &Parse<Json>) -> T { T::cast(self.to_node(root)).unwrap() }
}

impl Json {
  pub fn parse(text: &str) -> Parse<Json> {
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
