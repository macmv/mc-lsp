use crate::SyntaxKind;

use super::*;

pub fn json(p: &mut Parser) {
  let m = p.start();
  json::value(p);
  m.complete(p, SyntaxKind::JSON);
}
