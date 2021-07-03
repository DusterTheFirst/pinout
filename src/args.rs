use std::{path::PathBuf, str::FromStr};

use argh::FromArgs;

/// Synchronize your pinouts between firmware and electrical designs.
#[derive(FromArgs, Debug)]
pub struct Arguments {
    #[argh(positional)]
    pub netlist: PathBuf,
    /// reference of component to parse
    #[argh(positional)]
    pub reference: String,
    /// file to output generated constants
    #[argh(option, short = 'o')]
    pub output_file: PathBuf,
    /// language to generate constants in
    #[argh(option, short = 'L')]
    pub language: Language,
}

#[derive(Debug)]
pub enum Language {}

impl FromStr for Language {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        todo!()
    }
}
