use clap::{App, Arg};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Arguments {
    pub path: PathBuf,
    pub output: Option<PathBuf>,
}

pub fn parse_arguments() -> Arguments {
    let matches = App::new("Gm Docs Parsers")
        .version("0.1.0")
        .author("Jonathan Spira <jjspira@gmail.com>")
        .about("Parses Gms2.3 Documentation into Json over stdin/out")
        .version_short("v")
        .arg(
            Arg::with_name("INPUT_PATH")
                .takes_value(true)
                .required(true)
                .help(
                    "The path to the \"GameMaker_Langue\" folder within a Gms2.3 install.\
            If on your computer it is in a ZIP file, you will have to unzip\
            it and then give the path to that location.",
                ),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output_path")
                .takes_value(true)
                .help("The path to output to. If no path is provided, will output to stdout"),
        )
        .get_matches();

    Arguments {
        path: Path::new(matches.value_of("INPUT_PATH").unwrap()).to_owned(),
        output: matches.value_of("output").map(|o| Path::new(o).to_owned()),
    }
}
