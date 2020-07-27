pub use gm_docs_parser::*;
use std::path::Path;

mod parse_file;
mod parse_fnames;

fn main() {
    let fnames = parse_fnames::parse_fnames(Path::new("data"));

    for fname in fnames {
        parse_file::parse_function_file(&fname);
    }
}
