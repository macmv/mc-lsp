use std::error::Error;

use crate::{files::FileContent, global::GlobalState};

pub fn handle_open_text_document(
  global: &mut GlobalState,
  params: lsp_types::DidOpenTextDocumentParams,
) -> Result<(), Box<dyn Error>> {
  if let Some(path) = global.absolute_path(&params.text_document.uri) {
    let mut w = global.files.write();

    // TODO: We should discover sources here, not when the server starts. For now,
    // we only want this to do anything once the server has discovered all its
    // files.
    let file_id = match w.get_absolute(&path) {
      Some(id) => id,
      // We haven't indexed this file, so we don't care about it.
      None => return Ok(()),
    };

    w.write(file_id, FileContent::Json(params.text_document.text.clone()));
  }

  Ok(())
}

pub fn handle_change_text_document(
  global: &mut GlobalState,
  params: lsp_types::DidChangeTextDocumentParams,
) -> Result<(), Box<dyn Error>> {
  if let Some(path) = global.absolute_path(&params.text_document.uri) {
    let file_id = global.files.read().get_absolute(&path).ok_or("file not found")?;
    let FileContent::Json(file) = global.files.read().read(file_id) else {
      return Ok(());
    };

    let new_file = apply_changes(file.clone(), &params.content_changes);

    if file != new_file {
      global.files.write().write(file_id, FileContent::Json(new_file.clone()));
    }
  }

  Ok(())
}

pub fn handle_save_text_document(
  _global: &mut GlobalState,
  _params: lsp_types::DidSaveTextDocumentParams,
) -> Result<(), Box<dyn Error>> {
  // if let Some(path) = global.absolute_path(&params.text_document.uri) {
  // }

  Ok(())
}

fn apply_changes(
  mut file: String,
  changes: &[lsp_types::TextDocumentContentChangeEvent],
) -> String {
  for change in changes {
    match change.range {
      Some(range) => {
        let start = offset_of(&file, range.start);
        let end = offset_of(&file, range.end);

        file.replace_range(start..end, &change.text);
      }
      None => {
        file.replace_range(.., &change.text);
      }
    }
  }

  file
}

// TODO: Cache this somewhere.
fn offset_of(file: &str, pos: lsp_types::Position) -> usize {
  let mut offset = 0;

  for (i, line) in file.lines().enumerate() {
    if i == pos.line as usize {
      return offset + pos.character as usize;
    }

    offset += line.len() + 1;
  }

  offset
}
