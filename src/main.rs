pub use gm_docs_parser::*;
use std::path::Path;

mod parse_file;
mod parse_fnames;

fn main() {
    let _ = parse_fnames::parse_fnames(Path::new("data"));
    let x = Some(3);

    parse_file::parse_function_file(Path::new(
        "data/GameMaker_Language/GML_Reference/Drawing/Sprites_And_Tiles/draw_sprite.htm",
    ));
}
