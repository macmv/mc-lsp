use std::{fmt, str::FromStr};

/// A namespaced resource path.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Path {
  pub namespace: String,
  pub segments:  Vec<String>,
}

impl Path {
  pub fn new(namespace: String) -> Self { Path { namespace, segments: vec![] } }
}

impl FromStr for Path {
  type Err = ();

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let first = s.splitn(2, ':').next().ok_or(())?;
    let second = s.splitn(2, ':').nth(1);

    match second {
      Some(rest) => {
        let namespace = first.to_string();
        let segments = rest.split('/').map(|s| s.to_string()).collect();
        Ok(Path { namespace, segments })
      }
      None => {
        let namespace = "minecraft".to_string();
        let segments = first.split('/').map(|s| s.to_string()).collect();
        Ok(Path { namespace, segments })
      }
    }
  }
}

impl fmt::Display for Path {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if self.namespace == "minecraft" {
      write!(f, "{}", self.segments.join("/"))
    } else {
      write!(f, "{}:{}", self.namespace, self.segments.join("/"))
    }
  }
}

impl Path {
  pub fn strip_prefix(&self, prefix: &Path) -> Option<&[String]> {
    if self.namespace != prefix.namespace {
      return None;
    }

    self.segments.strip_prefix(prefix.segments.as_slice())
  }
}
