use crate::tests::check;

#[test]
fn strings() {
  check(
    r#" "foo" "#,
    expect![@r#"
      JSON
        WHITESPACE ' '
        STRING_VALUE
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
          NUMBER_VALUE
            NUMBER '1'
          COMMA ','
          WHITESPACE ' '
          NUMBER_VALUE
            NUMBER '2'
          COMMA ','
          WHITESPACE ' '
          NUMBER_VALUE
            NUMBER '3'
          COMMA ','
          WHITESPACE ' '
          BOOLEAN
            FALSE 'false'
          COMMA ','
          WHITESPACE ' '
          STRING_VALUE
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
            NUMBER_VALUE
              NUMBER '3'
            COMMA ','
          WHITESPACE ' '
          ELEMENT
            KEY
              STRING '"goodbye"'
            COLON ':'
            WHITESPACE ' '
            NUMBER_VALUE
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

#[test]
fn missing_quote_in_string() {
  check(
    r#"{ "p": b",  "t": 33  }"#,
    expect![@r#"
      JSON
        OBJECT
          OPEN_CURLY '{'
          WHITESPACE ' '
          ELEMENT
            KEY
              STRING '"p"'
            COLON ':'
            error: invalid character ' '
            error: expected value
            WHITESPACE 'b'
            STRING '",  "'
            error: invalid character 't'
            error: expected comma or end of object
          EOF '": 33'
    "#],
  );
}

#[test]
fn missing_key_quote() {
  check(
    r#"{
      "foo": 3,
      bar: 4,
      "baz": 5
    }"#,
    expect![@r#"
      JSON
        OBJECT
          OPEN_CURLY '{'
          WHITESPACE '\n      '
          ELEMENT
            KEY
              STRING '"foo"'
            COLON ':'
            WHITESPACE ' '
            NUMBER_VALUE
              NUMBER '3'
            COMMA ','
            error: invalid character '\n'
          WHITESPACE '      b'
          ELEMENT
            error: expected string
            error: invalid character 'a'
            error: expected colon
            error: expected value
            error: invalid character 'r'
            COLON ':'
            WHITESPACE ' '
            NUMBER '4'
            COMMA ','
          WHITESPACE '\n      '
          ELEMENT
            KEY
              STRING '"baz"'
            COLON ':'
            WHITESPACE ' '
            NUMBER_VALUE
              NUMBER '5'
          WHITESPACE '\n    '
          CLOSE_CURLY '}'
    "#],
  );
}
