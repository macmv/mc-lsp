use std::marker::PhantomData;

use rowan::GreenNode;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Parse<T> {
  green: GreenNode,
  _ty:   PhantomData<fn() -> T>,
}
