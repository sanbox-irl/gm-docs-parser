use serde_json::Value;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

const HELPDOCS_PATH: &str = "helpdocs_keywords.json";

pub fn parse_fnames(dir: &Path) {
    let path = dir.join(Path::new(HELPDOCS_PATH));

    let map: HashMap<String, PathBuf> =
        serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
    for (key, value) in map {
        if key == "Buffers" {
        }
    }
}
