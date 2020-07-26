use std::path::Path;

mod parse_fnames;

fn main() {
    parse_fnames::parse_fnames(Path::new("data"));
}
