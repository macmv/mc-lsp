use std::sync::Arc;

use mc_source::FileId;
use mc_syntax::{Json, Parse};

use crate::{diagnostic::Diagnostics, HirDatabase};

use super::{Blockstate, BlockstateSourceMap};

struct ModelValidator<'a> {
  db:         &'a dyn HirDatabase,
  blockstate: Arc<Blockstate>,
}

struct Validator<'a> {
  db:      &'a dyn HirDatabase,
  model:   &'a Blockstate,
  file_id: FileId,

  source_map:  &'a BlockstateSourceMap,
  json:        &'a Parse<Json>,
  diagnostics: &'a mut Diagnostics,
}

pub fn validate(
  db: &dyn HirDatabase,
  file_id: FileId,
  source_map: &BlockstateSourceMap,
  json: &Parse<Json>,
  diagnostics: &mut Diagnostics,
) {
  let blockstate = db.parse_blockstate(file_id);
  let mut validator = Validator { db, model: &blockstate, file_id, source_map, json, diagnostics };
  // validator.validate_model();
}
