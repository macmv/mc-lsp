use mc_source::TextRange;
use mc_syntax::SyntaxNode;

/// A collection of diagnostics in a single file.
#[derive(Debug, PartialEq, Eq)]
pub struct Diagnostics {
  diagnostics: Vec<Diagnostic>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
  pub span:    TextRange,
  pub message: String,
  pub level:   DiagnosticLevel,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiagnosticLevel {
  Error,
  Warn,
  Info,
}

pub trait Spanned {
  fn span(&self) -> TextRange;
}

impl Diagnostics {
  pub fn new() -> Self { Self { diagnostics: Vec::new() } }

  pub fn error(&mut self, node: impl Spanned, message: impl Into<String>) {
    self.diagnostics.push(Diagnostic::new_error(node.span(), message.into()));
  }
  pub fn warn(&mut self, node: impl Spanned, message: impl Into<String>) {
    self.diagnostics.push(Diagnostic::new_error(node.span(), message.into()));
  }
  pub fn info(&mut self, node: impl Spanned, message: impl Into<String>) {
    self.diagnostics.push(Diagnostic::new_error(node.span(), message.into()));
  }

  pub fn iter(&self) -> impl Iterator<Item = &Diagnostic> { self.diagnostics.iter() }
}

impl Diagnostic {
  pub fn new_error(span: TextRange, message: String) -> Self {
    Self { span, message, level: DiagnosticLevel::Error }
  }
  pub fn new_warn(span: TextRange, message: String) -> Self {
    Self { span, message, level: DiagnosticLevel::Warn }
  }
  pub fn new_info(span: TextRange, message: String) -> Self {
    Self { span, message, level: DiagnosticLevel::Info }
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
