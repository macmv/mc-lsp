pub mod highlight;

mod database;

#[allow(unused_imports)]
#[macro_use]
extern crate log;

use std::{panic::UnwindSafe, sync::Arc};

use database::{LineIndexDatabase, RootDatabase};
use highlight::Highlight;
use line_index::LineIndex;
use mc_source::{FileId, FileLocation, FileRange, SourceDatabase, Workspace};
use salsa::{Cancelled, ParallelDatabase};

pub struct AnalysisHost {
  db: RootDatabase,
}

/// A snapshot of analysis at a point in time.
pub struct Analysis {
  db: salsa::Snapshot<RootDatabase>,
}

pub type Cancellable<T> = Result<T, Cancelled>;

impl Default for AnalysisHost {
  fn default() -> Self { Self::new() }
}

impl AnalysisHost {
  pub fn new() -> Self {
    let mut db = RootDatabase::default();
    db.set_workspace(Default::default());
    AnalysisHost { db }
  }

  pub fn snapshot(&self) -> Analysis { Analysis { db: self.db.snapshot() } }

  pub fn set_workspace(&mut self, workspace: mc_source::Workspace) {
    self.db.set_workspace(workspace.into());
  }

  pub fn add_file(&mut self, file: FileId) { self.db.set_file_text(file, "".into()); }

  pub fn workspace(&self) -> Arc<Workspace> { self.db.workspace() }

  pub fn change(&mut self, change: Change) {
    self.db.set_file_text(change.file, change.text.into());
  }
}

pub struct Change {
  pub file: FileId,
  pub text: String,
}

impl ParallelDatabase for RootDatabase {
  fn snapshot(&self) -> salsa::Snapshot<Self> {
    salsa::Snapshot::new(RootDatabase { storage: self.storage.snapshot() })
  }
}

impl Analysis {
  pub fn completions(&self, _: FileLocation) -> Cancellable<Vec<()>> { self.with_db(|_| vec![]) }
  pub fn diagnostics(&self, _: FileId) -> Cancellable<Vec<()>> { self.with_db(|_| vec![]) }

  pub fn highlight(&self, file: FileId) -> Cancellable<Highlight> {
    self.with_db(|db| Highlight::from_ast(db, file))
  }

  pub fn definition_for_name(&self, _: FileLocation) -> Cancellable<Option<FileRange>> {
    self.with_db(|_| None)
  }

  pub fn references_for_name(&self, _: FileLocation) -> Cancellable<Vec<FileRange>> {
    self.with_db(|_| vec![])
  }

  pub fn line_index(&self, file: FileId) -> Cancellable<Arc<LineIndex>> {
    self.with_db(|db| db.line_index(file))
  }

  fn with_db<T>(&self, f: impl FnOnce(&RootDatabase) -> T + UnwindSafe) -> Cancellable<T> {
    Cancelled::catch(|| f(&self.db))
  }
}
