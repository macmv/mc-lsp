use std::str::FromStr;

/// A namespaced resource path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Path {
  pub namespace: String,
  pub segments:  Vec<String>,
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
