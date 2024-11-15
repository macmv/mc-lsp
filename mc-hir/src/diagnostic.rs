use mc_source::TextRange;
use mc_syntax::SyntaxNode;

/// A collection of diagnostics in a single file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostics {
  diagnostics: Vec<Diagnostic>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
  pub span:       TextRange,
  pub message:    String,
  pub severity:   Severity,
  pub hints:      Vec<String>,
  pub suggestion: Option<Suggestion>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Suggestion {
  pub title: String,

  pub range:   TextRange,
  pub replace: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
  Error,
  Warn,
  Info,
}

pub trait Spanned {
  fn span(&self) -> TextRange;
}

impl Diagnostics {
  pub fn new() -> Self { Self { diagnostics: Vec::new() } }

  pub fn push(&mut self, diagnostic: Diagnostic) { self.diagnostics.push(diagnostic) }
  pub fn extend(&mut self, diagnostics: &Diagnostics) {
    self.diagnostics.extend_from_slice(&diagnostics.diagnostics)
  }

  pub fn error(&mut self, node: impl Spanned, message: impl Into<String>) -> &mut Diagnostic {
    self.diagnostics.push(Diagnostic::new_error(node.span(), message.into()));
    self.diagnostics.last_mut().unwrap()
  }
  pub fn warn(&mut self, node: impl Spanned, message: impl Into<String>) -> &mut Diagnostic {
    self.diagnostics.push(Diagnostic::new_warn(node.span(), message.into()));
    self.diagnostics.last_mut().unwrap()
  }
  pub fn info(&mut self, node: impl Spanned, message: impl Into<String>) -> &mut Diagnostic {
    self.diagnostics.push(Diagnostic::new_info(node.span(), message.into()));
    self.diagnostics.last_mut().unwrap()
  }

  pub fn iter(&self) -> impl Iterator<Item = &Diagnostic> { self.diagnostics.iter() }
}

impl Diagnostic {
  pub fn new(span: TextRange, message: String, severity: Severity) -> Self {
    Diagnostic { span, message, severity, hints: vec![], suggestion: None }
  }

  pub fn new_error(span: TextRange, message: String) -> Self {
    Diagnostic::new(span, message, Severity::Error)
  }
  pub fn new_warn(span: TextRange, message: String) -> Self {
    Diagnostic::new(span, message, Severity::Warn)
  }
  pub fn new_info(span: TextRange, message: String) -> Self {
    Diagnostic::new(span, message, Severity::Info)
  }

  pub fn hint(&mut self, message: impl Into<String>) -> &mut Self {
    self.hints.push(message.into());
    self
  }

  pub fn suggest_remove(&mut self, title: impl Into<String>, span: TextRange) -> &mut Self {
    self.suggestion = Some(Suggestion::remove(title, span));
    self
  }
}

impl Suggestion {
  pub fn remove(title: impl Into<String>, range: TextRange) -> Self {
    Suggestion { title: title.into(), range, replace: String::new() }
  }
}

impl Spanned for TextRange {
  fn span(&self) -> TextRange { *self }
}

impl Spanned for SyntaxNode {
  fn span(&self) -> TextRange { self.text_range() }
}

impl<T: Spanned> Spanned for &T {
  fn span(&self) -> TextRange { (*self).span() }
}
