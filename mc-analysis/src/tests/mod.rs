use std::sync::Arc;

use mc_source::{File, FileId, FileType, Namespace, SourceDatabase, Workspace};

use crate::database::RootDatabase;

mod completion;

pub fn test_db() -> RootDatabase {
  let mut db = RootDatabase::default();

  const BLOCK_MODEL: FileId = FileId::new_raw(0);
  const FOO_MODEL: FileId = FileId::new_raw(1);
  const BAR_TEXTURE: FileId = FileId::new_raw(2);

  db.set_workspace(Arc::new(Workspace {
    namespaces: vec![
      Namespace {
        name:  "minecraft".to_string(),
        files: vec![File {
          id:   BLOCK_MODEL,
          ty:   FileType::Model,
          path: "minecraft:models/block/block.json".parse().unwrap(),
        }],
      },
      Namespace {
        name:  "test".to_string(),
        files: vec![
          File {
            id:   FOO_MODEL,
            ty:   FileType::Model,
            path: "test:models/block/foo.json".parse().unwrap(),
          },
          // NB: File text is undefined for this file.
          File {
            id:   BAR_TEXTURE,
            ty:   FileType::Model,
            path: "test:textures/blocks/bar.png".parse().unwrap(),
          },
        ],
      },
    ],
  }));

  db.set_file_text(FileId::new_raw(0), "{}".into());
  db.set_file_text(FileId::new_raw(1), "{}".into());

  db.set_file_type(FileId::new_raw(0), FileType::Model);
  db.set_file_type(FileId::new_raw(1), FileType::Model);

  db
}

#[test]
fn test_db_works() {
  let db = test_db();

  assert_eq!(db.file_text(FileId::new_raw(0)).as_ref(), "{}");
  assert_eq!(db.file_text(FileId::new_raw(1)).as_ref(), "{}");

  assert_eq!(db.file_type(FileId::new_raw(0)), FileType::Model);
  assert_eq!(db.file_type(FileId::new_raw(1)), FileType::Model);
}
