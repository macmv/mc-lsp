use std::marker::PhantomData;

mod ast;

pub use ast::Json;
use rowan::{GreenNode, SyntaxNode};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Parse<T> {
  green: GreenNode,
  _ty:   PhantomData<fn() -> T>,
}

impl Json {
  pub fn parse(text: &str) -> Parse<Json> {
    // TODO
    let (green, errors) = parse::parse_text(text);
    let root = SyntaxNode::new_root(green.clone());

    // assert_eq!(root.kind(), SyntaxKind::SOURCE_FILE);
    Parse { green, _ty: PhantomData }
  }
}
