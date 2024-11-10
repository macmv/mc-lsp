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
