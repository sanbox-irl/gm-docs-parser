use clap::Parser;
use std::path::PathBuf;

/// A CLI intended for use by humans and machines to build GameMakerStudio 2 projects.
#[derive(Parser, Debug)]
pub struct InputOpts {
    /// The path to the GameMaker_Language folder within a 2.3 install.
    /// If oin your computer it is in a ZIP file, you will have to unzip it and then give a path
    /// to that location.
    pub input_path: PathBuf,

    /// If set to true, then we won't emit anything. This basically makes it a linter for itself.
    #[clap(short, long)]
    pub do_not_emit: bool,
}
