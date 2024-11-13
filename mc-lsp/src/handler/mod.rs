use lsp_types::request::Request;
use serde::{Deserialize, Serialize};

pub mod notification;
pub mod request;

pub enum CanonicalModel {}

impl Request for CanonicalModel {
  type Params = CanonicalModelParams;
  type Result = Option<CanonicalModelResponse>;
  const METHOD: &'static str = "mc-lsp/canonicalModel";
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CanonicalModelParams {
  pub uri: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CanonicalModelResponse {
  pub model: serde_json::Value,
}
