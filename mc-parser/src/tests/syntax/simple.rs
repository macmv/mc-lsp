use crate::tests::check;

#[test]
fn strings() {
  check(
    r#" "foo" "#,
    expect![@r#"
      JSON
        WHITESPACE ' '
        STRING '"foo"'
    "#],
  );
}

#[test]
fn booleans() {
  check(
    r#"true"#,
    expect![@r#"
      JSON
        BOOLEAN
          TRUE 'true'
    "#],
  );
}

#[test]
fn arrays() {
  check(
    r#"[1, 2, 3, false, "hello"]"#,
    expect![@r#"
      JSON
        ARRAY
          OPEN_BRACKET '['
          NUMBER '1'
          COMMA ','
          WHITESPACE ' '
          NUMBER '2'
          COMMA ','
          WHITESPACE ' '
          NUMBER '3'
          COMMA ','
          WHITESPACE ' '
          BOOLEAN
            FALSE 'false'
          COMMA ','
          WHITESPACE ' '
          STRING '"hello"'
          CLOSE_BRACKET ']'
    "#],
  );
}

#[test]
fn objects() {
  check(
    r#"{ "hello": 3, "goodbye": 4, "foo": null }"#,
    expect![@r#"
      JSON
        OBJECT
          OPEN_CURLY '{'
          WHITESPACE ' '
          ELEMENT
            KEY
              STRING '"hello"'
            COLON ':'
            WHITESPACE ' '
            NUMBER '3'
            COMMA ','
          WHITESPACE ' '
          ELEMENT
            KEY
              STRING '"goodbye"'
            COLON ':'
            WHITESPACE ' '
            NUMBER '4'
            COMMA ','
          WHITESPACE ' '
          ELEMENT
            KEY
              STRING '"foo"'
            COLON ':'
            WHITESPACE ' '
            NULL 'null'
          WHITESPACE ' '
          CLOSE_CURLY '}'
    "#],
  );
}
