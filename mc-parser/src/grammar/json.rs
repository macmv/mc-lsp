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

    // test ok
    // null
    T![null] => p.eat(T![null]),

    // test ok
    // { "hi": 3 }
    T!['{'] => object(p),
    // test ok
    // ["hi", 3]
    T!['['] => array(p),

    _ => p.error("expected value"),
  }

  match p.current() {
    T![,] | T!['}'] | T![']'] | SyntaxKind::EOF => {}
    _ => p.error("expected end of value"),
  }
}

fn object(p: &mut Parser) {
  let m = p.start();
  p.bump();

  while !p.at(T!['}']) {
    let m = p.start();
    if p.at(T![string]) {
      let m = p.start();
      p.bump();
      m.complete(p, SyntaxKind::KEY);
    } else {
      p.error("expected string");
    }

    if p.at(T![:]) {
      p.eat(T![:]);
    } else {
      p.error("expected colon");
    }

    value(p);

    match p.current() {
      T![,] => {
        p.bump();
      }
      T!['}'] => {}
      _ => p.error("expected comma or end of object"),
    }

    m.complete(p, SyntaxKind::ELEMENT);
  }

  p.bump();
  m.complete(p, SyntaxKind::OBJECT);
}

fn array(p: &mut Parser) {
  let m = p.start();
  p.bump();

  while !p.at(T![']']) {
    value(p);

    match p.current() {
      T![,] => {
        p.bump();
      }
      T![']'] => {}
      _ => p.error("expected comma or end of array"),
    }
  }

  p.bump();
  m.complete(p, SyntaxKind::ARRAY);
}
