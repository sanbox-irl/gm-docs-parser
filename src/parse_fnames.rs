use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

const HELPDOCS_PATH: &str = "helpdocs_keywords.json";

pub fn parse_fnames(dir: &Path) -> Vec<PathBuf> {
    let path = dir.join(Path::new(HELPDOCS_PATH));

    let map: HashMap<String, PathBuf> =
        serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();

    map.into_iter()
        .filter_map(|(name, fpath)| {
            if name.contains(char::is_uppercase) {
                return None;
            }
            if fpath
                .to_string_lossy()
                .contains("GameMaker_Language/GML_Reference")
                == false
            {
                return None;
            }

            let mut path = dir.join(&fpath);
            path.set_extension("htm");

            Some(path)
        })
        .collect()
}
