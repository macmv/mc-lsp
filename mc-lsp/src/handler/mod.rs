use lsp_types::{request::Request, Url};
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
  pub uri: Url,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CanonicalModelResponse {
  pub model: mc_message::Model,
}
