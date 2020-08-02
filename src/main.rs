#![allow(clippy::bool_comparison)]

pub use gm_docs_parser::*;
use std::path::Path;

mod markdown;
mod parse_constants;
mod parse_file;
mod parse_fnames;
pub use markdown::Markdown;

fn main() {
    env_logger::init();

    let base_path = Path::new("data");
    let fnames = parse_fnames::parse_fnames(base_path);

    let mut functions = vec![];
    let mut variables = vec![];

    for fname in fnames {
        if let Some(success) = parse_file::parse_function_file(&fname) {
            match success {
                parse_file::DocEntry::Function(v) => {
                    functions.push(v);
                }
                parse_file::DocEntry::Variable(v) => {
                    variables.push(v);
                }
            }
        }
    }

    let mut constants: Vec<GmManualConstant> = vec![];
    parse_constants::parse_constants(base_path, &mut constants).unwrap();

    let gm_manual = GmManual {
        functions: functions.into_iter().map(|v| (v.name.clone(), v)).collect(),
        variables: variables.into_iter().map(|v| (v.name.clone(), v)).collect(),
        constants: constants.into_iter().map(|v| (v.name.clone(), v)).collect(),
    };

    let st = serde_json::to_string_pretty(&gm_manual).unwrap();
    println!("{}", st);
}
