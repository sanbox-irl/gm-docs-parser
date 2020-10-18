#![allow(clippy::bool_comparison)]

pub use gm_docs_parser::*;
mod arg;
mod markdown;
mod parse_constants;
mod parse_file;
mod parse_fnames;
pub use markdown::Markdown;

use clap::Clap;

fn main() {
    env_logger::init();
    let arguments: arg::InputOpts = arg::InputOpts::parse();
    let fnames = parse_fnames::parse_fnames(arguments.input_path);

    let mut gm_manual = GmManual::default();
    for fname in fnames {
        if let Some(success) = parse_file::parse_function_file(&fname) {
            match success {
                parse_file::DocEntry::Function(v) => {
                    gm_manual.functions.insert(v.name.clone(), v);
                }
                parse_file::DocEntry::Variable(v) => {
                    gm_manual.variables.insert(v.name.clone(), v);
                }
            }
        }
    }

    let base_path = parse_fnames::base_path();
    parse_constants::parse_constants(&base_path, &mut gm_manual.constants).unwrap();

    let st = serde_json::to_string_pretty(&gm_manual).unwrap();
    println!("{}", st.replace('\u{a0}', " "));
}
