use once_cell::sync::Lazy;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
};
use url::Url;

const HELPDOCS_PATH: &str = "helpdocs_keywords.json";

static BASE_PATH: Lazy<Mutex<PathBuf>> = Lazy::new(Default::default);

pub fn base_path() -> PathBuf {
    (*BASE_PATH.lock().unwrap()).clone()
}

pub fn parse_fnames(dir: PathBuf) -> BTreeSet<PathBuf> {
    let mut thing = BASE_PATH.lock().unwrap();
    *thing = dir.clone();

    let path = dir.join(Path::new(HELPDOCS_PATH));

    let map: BTreeMap<String, PathBuf> =
        serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();

    map.into_iter()
        .filter_map(|(name, fpath)| {
            if name.contains(char::is_uppercase)
                || fpath
                    .file_name()
                    .map(|fname| fname.to_string_lossy().contains(char::is_uppercase))
                    .unwrap_or_default()
            {
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

pub fn convert_to_url(path_to_strip: &Path) -> Url {
    let output = path_to_strip
        .strip_prefix(&*BASE_PATH.lock().unwrap())
        .unwrap();

    Url::parse(&format!(
        "https://manual.yoyogames.com/{}",
        output.to_str().unwrap()
    ))
    .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn convert_back() {
        let path = Path::new(
            "data/GameMaker_Language/GML_Reference/Game_Input/Mouse_Input/mouse_clear.htm",
        );

        let output = path.strip_prefix("data/").unwrap();

        assert_eq!(
            output,
            Path::new("GameMaker_Language/GML_Reference/Game_Input/Mouse_Input/mouse_clear.htm")
        )
    }
}
