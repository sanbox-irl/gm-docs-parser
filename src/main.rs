#![allow(clippy::bool_comparison)]

pub use gm_docs_parser::*;
use std::path::Path;

mod parse_file;
mod parse_fnames;

fn main() {
    env_logger::init();

    let fnames = parse_fnames::parse_fnames(Path::new("data"));

    for fname in fnames {
        parse_file::parse_function_file(&fname);
    }
}
