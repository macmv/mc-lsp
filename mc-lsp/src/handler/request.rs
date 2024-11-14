use std::{error::Error, path::Path, sync::Arc};

use line_index::LineIndex;
use lsp_types::{SemanticTokenModifier, SemanticTokenType};
use mc_analysis::{
  completion::CompletionKind,
  highlight::{Highlight, HighlightKind},
};
use mc_source::{FileId, FileLocation, TextRange, TextSize};

use crate::global::GlobalStateSnapshot;

/// Converts file positions to LSP positions.
struct LspConverter {
  line_index: Arc<LineIndex>,
}

impl LspConverter {
  pub fn from_pos(
    snap: &GlobalStateSnapshot,
    pos: lsp_types::TextDocumentPositionParams,
  ) -> Result<(FileLocation, Self), Box<dyn Error>> {
    let pos = file_position(&snap, pos)?;

    Ok((pos, LspConverter::new(snap, pos.file)?))
  }

  pub fn new(snap: &GlobalStateSnapshot, file: FileId) -> Result<Self, Box<dyn Error>> {
    Ok(Self { line_index: snap.analysis.line_index(file)? })
  }

  pub fn pos(&self, index: TextSize) -> lsp_types::Position {
    let pos = self.line_index.line_col(index);
    lsp_types::Position { line: pos.line, character: pos.col }
  }

  pub fn range(&self, range: TextRange) -> lsp_types::Range {
    let start = self.pos(range.start());
    let end = self.pos(range.end());

    lsp_types::Range { start, end }
  }
}

pub fn handle_completion(
  snap: GlobalStateSnapshot,
  params: lsp_types::CompletionParams,
) -> Result<Option<lsp_types::CompletionResponse>, Box<dyn Error>> {
  if let Some(_) = snap.absolute_path(&params.text_document_position.text_document.uri) {
    let (cursor_pos, _) = LspConverter::from_pos(&snap, params.text_document_position)?;
    let completions = snap.analysis.completions(cursor_pos)?;

    Ok(Some(lsp_types::CompletionResponse::Array(
      completions
        .into_iter()
        .map(|c| lsp_types::CompletionItem {
          label: c.label,
          label_details: Some(lsp_types::CompletionItemLabelDetails {
            detail:      None,
            description: Some(c.description),
          }),
          kind: Some(match c.kind {
            CompletionKind::Model => lsp_types::CompletionItemKind::CLASS,
            CompletionKind::Texture => lsp_types::CompletionItemKind::TEXT,
            CompletionKind::Namespace => lsp_types::CompletionItemKind::MODULE,
          }),

          insert_text: Some(c.insert),
          command: if c.retrigger {
            Some(lsp_types::Command {
              command:   "editor.action.triggerSuggest".to_owned(),
              arguments: None,
              title:     "Re-trigger completions".to_owned(),
            })
          } else {
            None
          },
          ..Default::default()
        })
        .collect(),
    )))
  } else {
    Ok(None)
  }
}

pub fn handle_semantic_tokens_full(
  snap: GlobalStateSnapshot,
  params: lsp_types::SemanticTokensParams,
) -> Result<Option<lsp_types::SemanticTokensResult>, Box<dyn Error>> {
  if let Some(path) = snap.absolute_path(&params.text_document.uri) {
    let file_id = snap.files.read().get_absolute(&path).ok_or("file not found")?;
    let highlight = snap.analysis.highlight(file_id)?;

    let tokens = to_semantic_tokens(snap, file_id, &highlight)?;

    Ok(Some(lsp_types::SemanticTokensResult::Tokens(lsp_types::SemanticTokens {
      data:      tokens,
      result_id: None,
    })))
  } else {
    Ok(None)
  }
}

pub fn handle_goto_definition(
  snap: GlobalStateSnapshot,
  params: lsp_types::GotoDefinitionParams,
) -> Result<Option<lsp_types::GotoDefinitionResponse>, Box<dyn Error>> {
  let (cursor_pos, _) = LspConverter::from_pos(&snap, params.text_document_position_params)?;
  let definition = snap.analysis.definition_for_name(cursor_pos)?;

  if let Some(def) = definition {
    let files = snap.files.read();

    Ok(Some(lsp_types::GotoDefinitionResponse::Scalar(lsp_types::Location::new(
      lsp_types::Url::parse(&format!("file://{}", files.id_to_absolute_path(def.file).display()))
        .unwrap(),
      match def.range {
        Some(range) => {
          let converter = LspConverter::new(&snap, def.file)?;
          converter.range(range)
        }
        None => lsp_types::Range {
          start: lsp_types::Position::new(0, 0),
          end:   lsp_types::Position::new(0, 0),
        },
      },
    ))))
  } else {
    Ok(None)
  }
}

pub fn handle_document_highlight(
  snap: GlobalStateSnapshot,
  params: lsp_types::DocumentHighlightParams,
) -> Result<Option<Vec<lsp_types::DocumentHighlight>>, Box<dyn Error>> {
  let (cursor_pos, converter) =
    LspConverter::from_pos(&snap, params.text_document_position_params)?;
  let definition = snap.analysis.definition_for_name(cursor_pos)?;
  let refs = snap.analysis.references_for_name(cursor_pos)?;

  if let Some(def) = definition {
    if cursor_pos.file != def.file {
      return Ok(None);
    }

    let def_highlight = lsp_types::DocumentHighlight {
      range: converter.range(def.range.unwrap()),
      kind:  Some(lsp_types::DocumentHighlightKind::WRITE),
    };

    let refs_highlight = refs.into_iter().map(|r| lsp_types::DocumentHighlight {
      range: converter.range(r.range.unwrap()),
      kind:  Some(lsp_types::DocumentHighlightKind::READ),
    });

    Ok(Some([def_highlight].into_iter().chain(refs_highlight).collect()))
  } else {
    Ok(None)
  }
}

pub fn handle_hover(
  _snap: GlobalStateSnapshot,
  _params: lsp_types::HoverParams,
) -> Result<Option<lsp_types::Hover>, Box<dyn Error>> {
  Ok(None)
  /*
  let (pos, converter) = LspConverter::from_pos(&snap, params.text_document_position_params)?;
  let def = snap.analysis.definition_for_name(pos)?;
  let ty = snap.analysis.type_at(pos)?;

  let range = def.map(|(_, pos)| converter.range(pos.range));

  Ok(Some(lsp_types::Hover {
    range,
    contents: lsp_types::HoverContents::Scalar(lsp_types::MarkedString::String(match ty {
      Some(ty) => ty.to_string(),
      None => "unknown type".to_string(),
    })),
  }))
  */
}

pub fn handle_canonical_model(
  snap: GlobalStateSnapshot,
  params: super::CanonicalModelParams,
) -> Result<Option<super::CanonicalModelResponse>, Box<dyn Error>> {
  let path = Path::new(params.uri.path());
  if let Some(file) = snap.files.read().get_absolute(path) {
    let model = snap.analysis.canonical_model(file)?;

    Ok(Some(super::CanonicalModelResponse { model }))
  } else {
    Ok(None)
  }
}

struct TokenModifier {
  stat: bool,
}

impl TokenModifier {
  pub fn all() -> Vec<SemanticTokenModifier> { vec![SemanticTokenModifier::new("static")] }

  pub fn from_kind(_: HighlightKind) -> Self { Self { stat: false } }

  pub fn encode(&self) -> u32 {
    let mut bits = 0;

    if self.stat {
      bits |= 1;
    }

    bits
  }
}

pub fn semantic_tokens_legend() -> lsp_types::SemanticTokensLegend {
  fn token_type(kind: HighlightKind) -> SemanticTokenType {
    match kind {
      HighlightKind::Keyword => SemanticTokenType::new("keyword"),
      HighlightKind::UnknownKey => SemanticTokenType::new("string"),
      HighlightKind::Number | HighlightKind::Boolean | HighlightKind::Null => {
        SemanticTokenType::new("number")
      }
      HighlightKind::Texture => SemanticTokenType::new("string"),
      HighlightKind::Variable => SemanticTokenType::new("variable"),
    }
  }

  lsp_types::SemanticTokensLegend {
    token_types:     HighlightKind::iter().map(token_type).collect(),
    token_modifiers: TokenModifier::all(),
  }
}

fn to_semantic_tokens(
  snap: GlobalStateSnapshot,
  file: FileId,
  highlight: &Highlight,
) -> Result<Vec<lsp_types::SemanticToken>, Box<dyn Error>> {
  let line_index = snap.analysis.line_index(file)?;

  let mut tokens = Vec::new();

  let mut line = 0;
  let mut col = 0;

  for h in highlight.tokens.iter() {
    let range = h.range;

    let pos = line_index.line_col(range.start());

    let delta_line = pos.line - line;
    if delta_line != 0 {
      col = 0;
    }
    let delta_start = pos.col - col;

    line = pos.line;
    col = pos.col;

    tokens.push(lsp_types::SemanticToken {
      delta_line,
      delta_start,
      length: (range.end() - range.start()).into(),
      token_type: h.kind as u32,
      token_modifiers_bitset: TokenModifier::from_kind(h.kind).encode(),
    });
  }

  Ok(tokens)
}

fn file_position(
  snap: &GlobalStateSnapshot,
  pos: lsp_types::TextDocumentPositionParams,
) -> Result<FileLocation, Box<dyn Error>> {
  let files = snap.files.read();

  let path = Path::new(pos.text_document.uri.path());
  let file_id = files.get_absolute(path).ok_or("file not found")?;

  let index = snap.analysis.line_index(file_id)?;

  match index.offset(line_index::LineCol { line: pos.position.line, col: pos.position.character }) {
    Some(index) => Ok(FileLocation { file: file_id, index }),
    None => Err("position not found".into()),
  }
}
