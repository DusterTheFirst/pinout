use std::{path::PathBuf, str::FromStr};

use argh::FromArgs;
use strum::{Display, EnumVariantNames, VariantNames};

/// Synchronize your pinouts between firmware and electrical designs.
#[derive(FromArgs, Debug)]
pub struct Arguments {
    #[argh(positional)]
    pub netlist: PathBuf,
    /// reference of component to parse
    #[argh(option, long = "ref")]
    pub reference: String,
    /// file to output generated constants
    #[argh(option, short = 'o', long = "output")]
    pub output_file: PathBuf,
    /// language to generate constants in
    #[argh(option, long = "lang")]
    pub language: Language,
}

#[derive(Debug, Display, EnumVariantNames)]
pub enum Language {
    C,
    Cpp,
    Rust,
}

impl FromStr for Language {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "c" => Language::C,
            "cpp" | "cxx" | "c++" => Language::Cpp,
            "rust" => Language::Rust,
            _ => {
                return Err(format!(
                    "unsupported language, supported languages are: {}",
                    Language::VARIANTS.join(", ")
                ))
            }
        })
    }
}
