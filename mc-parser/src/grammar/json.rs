use crate::{Parser, SyntaxKind, T};

pub fn value(p: &mut Parser) {
  match p.current() {
    // test ok
    // "hi"
    //
    // test ok
    // 3
    T![string] | T![number] => {
      p.bump();
    }

    // test ok
    // true
    T![true] | T![false] => {
      let m = p.start();
      p.bump();
      m.complete(p, SyntaxKind::BOOLEAN);
    }

    // T!['{'] => object(p),
    // T!['['] => array(p),

    // T![true] | T![false] => boolean(p),
    _ => p.error("expected value"),
  }

  match p.current() {
    T!['}'] | T![']'] | SyntaxKind::EOF => {}
    _ => p.error("expected end of value"),
  }
}
