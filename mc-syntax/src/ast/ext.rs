use crate::ast;

impl ast::Value {
  pub fn as_str(&self) -> Option<String> {
    match self {
      ast::Value::StringValue(s) => Some(s.parse_text()),
      _ => None,
    }
  }

  pub fn as_i64(&self) -> Option<i64> {
    match self {
      ast::Value::NumberValue(s) => s.syntax.text().to_string().parse().ok(),
      _ => None,
    }
  }

  pub fn as_bool(&self) -> Option<bool> {
    match self {
      ast::Value::Boolean(s) => Some(s.syntax.text().to_string() == "true"),
      _ => None,
    }
  }

  pub fn as_object(&self) -> Option<ast::Object> {
    match self {
      ast::Value::Object(o) => Some(o.clone()),
      _ => None,
    }
  }
}

impl ast::Key {
  pub fn parse_text(&self) -> String { parse_text(&self.syntax.text().to_string()) }
}
impl ast::StringValue {
  pub fn parse_text(&self) -> String { parse_text(&self.syntax.text().to_string()) }
}

fn parse_text(text: &str) -> String {
  let mut out = String::new();

  let mut in_escape = false;
  for c in text.strip_prefix("\"").unwrap().strip_suffix("\"").unwrap().chars() {
    match c {
      '\\' if !in_escape => in_escape = true,
      '\\' if in_escape => {
        out.push('\\');
        in_escape = false;
      }
      'n' if in_escape => {
        out.push('\n');
        in_escape = false;
      }
      'r' if in_escape => {
        out.push('\r');
        in_escape = false;
      }
      't' if in_escape => {
        out.push('\t');
        in_escape = false;
      }
      _ if in_escape => {
        out.push(c);
        in_escape = false;
      }

      _ => out.push(c),
    }
  }

  out
}
