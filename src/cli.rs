use std::path::PathBuf;

use argh::FromArgs;

#[derive(FromArgs, Debug, Eq, PartialEq)]
#[argh(description = "Relativize m3u playlist paths")]
pub struct Args {
    #[argh(
        option,
        description = "number of ending path segments to match (default: 1)",
        short = 'd',
        default = "default_depth()"
    )]
    pub depth: u8,
    #[argh(
        switch,
        description = "whether or not to be strict about extensions (default: false)",
        short = 's',
        long = "strict"
    )]
    pub strict_extension: bool,
    #[argh(positional, description = "path")]
    pub path: PathBuf,
}

fn default_depth() -> u8 {
    1
}
