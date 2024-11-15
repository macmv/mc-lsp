use mc_source::{FileLocation, SourceDatabase, TextSize};
use mc_test::{expect, Expect};

use super::FOO_MODEL;

fn complete(input: &str, expect: Expect) {
  let mut db = super::test_db();

  let cursor = input.find('|').unwrap();
  let input = input[..cursor].to_string() + &input[cursor + 1..];

  db.set_file_text(FOO_MODEL, input.into());

  let completions = crate::completion::completions(
    &db,
    FileLocation { file: FOO_MODEL, index: TextSize::from(cursor as u32) },
  );

  expect
    .assert_eq(&columns(completions.iter().map(|c| [c.label.as_str(), c.description.as_str()])));
}

fn columns<'a, const N: usize>(iter: impl Iterator<Item = [&'a str; N]> + Clone) -> String {
  let mut maximums = [0; N];
  for row in iter.clone() {
    for (i, cell) in row.iter().enumerate() {
      maximums[i] = maximums[i].max(cell.len());
    }
  }

  let mut out = String::new();

  for row in iter {
    for (i, cell) in row.iter().enumerate() {
      out.push_str(cell);
      if i != row.len() - 1 {
        out.push_str(&" ".repeat(maximums[i] - cell.len()));
        out.push_str("  ");
      }
    }
    out.push_str("\n");
  }

  out
}

#[test]
fn complete_parent() {
  complete(
    r#"{
      "parent": "|",
    }"#,
    expect![@r#"
      block/block     minecraft:block/block
      test:block/foo  test:block/foo
    "#],
  );
}

#[test]
fn complete_keys() {
  complete(
    r#"{
      |
    }"#,
    expect![@r#"
      "parent"    parent
      "textures"  textures
      "elements"  elements
    "#],
  );

  complete(
    r#"{
      "elements": [
        {
          |
        }
      ]
    }"#,
    expect![@r#"
      "from"      from
      "to"        to
      "rotation"  rotation
      "faces"     faces
    "#],
  );
}

#[test]
fn excludes_existing_keys() {
  complete(
    r#"{
      |
      "elements": [{}]
    }"#,
    expect![@r#"
      "parent"    parent
      "textures"  textures
    "#],
  );

  complete(
    r#"{
      "foo|": 0,
      "elements": [{}]
    }"#,
    expect![@r#"
      "parent"    parent
      "textures"  textures
    "#],
  );
}
