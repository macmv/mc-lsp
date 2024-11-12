use std::{env, path::PathBuf};

pub fn extract_jar() {
  let home = env::var("HOME").unwrap();

  // NB: This is just the client jar. We could pull this down from mojang, but we
  // can also just assume that forge gradle has pulled it down, and unzip it
  // locally.
  //
  // TODO: Needs way, way more error handling.
  let mut path = PathBuf::new();
  path.push(&home);
  path.extend(&[
    ".gradle",
    "caches",
    "forge_gradle",
    "minecraft_repo",
    "versions",
    "1.12.2",
    "client.jar",
  ]);

  let mut destination = PathBuf::new();
  destination.push(&home);
  destination.extend(&[".cache", "mclsp", "minecraft", "versions", "1.12.2", "client"]);

  std::fs::create_dir_all(&destination).unwrap();

  let mut archive = zip::ZipArchive::new(std::fs::File::open(path).unwrap()).unwrap();

  for i in 0..archive.len() {
    let name = archive.name_for_index(i).unwrap();
    let mut path = destination.clone();
    path.push(name);

    // We only care about model files.
    if !name.ends_with(".json") {
      continue;
    }

    std::fs::create_dir_all(&path.parent().unwrap()).unwrap();

    let mut input = archive.by_index(i).unwrap();
    let mut out = std::fs::File::create(&path).unwrap();
    std::io::copy(&mut input, &mut out).unwrap();
  }
}

#[test]
fn test_extract_jar() { extract_jar(); }
