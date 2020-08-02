#![allow(clippy::bool_comparison)]

pub use gm_docs_parser::*;
use std::path::Path;

mod parse_file;
mod parse_fnames;

fn main() {
    env_logger::init();

    // let fnames = parse_fnames::parse_fnames(Path::new("data"));

    let mut output = vec![];
    let mut constants = vec![];
    // for fname in fnames {
    if let Some(success) = parse_file::parse_function_file(
        &Path::new("data/GameMaker_Language/GML_Reference/Game_Input/Mouse_Input/mouse_clear.htm"),
        &mut constants,
    ) {
        output.push(success);
        // }
    }

    println!("{:#?}", output);

    let x = NameDescription {
        name: "hello".to_string(),
        description: "one".to_string(),
        secondary_description: Some(NameDescription {
            name: "two".to_string(),
            description: "three".to_string(),
            secondary_description: None,
        }),
    };

    // let st = serde_json::to_string_pretty(&output).unwrap();
    // std::fs::write("out.json", st).unwrap();
}
