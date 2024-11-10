#![allow(dead_code)]

use mc_parser::SyntaxKind;
use rowan::Language;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Mc {}
impl Language for Mc {
  type Kind = SyntaxKind;

  fn kind_from_raw(raw: rowan::SyntaxKind) -> SyntaxKind { SyntaxKind::from(raw.0) }

  fn kind_to_raw(kind: SyntaxKind) -> rowan::SyntaxKind { rowan::SyntaxKind(kind.into()) }
}

pub type SyntaxNode = rowan::SyntaxNode<Mc>;
pub type SyntaxToken = rowan::SyntaxToken<Mc>;
pub type SyntaxElement = rowan::SyntaxElement<Mc>;
pub type SyntaxNodeChildren = rowan::SyntaxNodeChildren<Mc>;
pub type SyntaxElementChildren = rowan::SyntaxElementChildren<Mc>;
pub type PreorderWithTokens = rowan::api::PreorderWithTokens<Mc>;
pub type NodeOrToken = rowan::NodeOrToken<SyntaxNode, SyntaxToken>;
