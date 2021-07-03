use anyhow::Context;
use args::Language;
use netlist::Netlist;
use regex::Regex;
use std::fs;

use crate::args::Arguments;

mod args;
mod netlist;
mod sexpr;

fn main() -> anyhow::Result<()> {
    let args: Arguments = argh::from_env();

    dbg!(&args);

    eprintln!("Loading netlist {:?}", args.netlist);

    let sexpr = {
        let netlist_file = fs::read_to_string(&args.netlist)
            .context("Failed to open the provided netlist file")?;

        let regex = Regex::new("(  )?\\(tstamp .*?\\)").unwrap();

        lexpr::from_str(&regex.replace_all(&netlist_file, ""))
            .context("Failed to parse the given netlist file")?
    };

    let netlist = Netlist::new(&sexpr);

    eprintln!(
        "Found sheet `{}` {} by {}",
        netlist.title, netlist.rev, netlist.company,
    );

    eprintln!(
        "Generating {} file `{}`",
        args.language,
        args.output_file.to_string_lossy(),
    );

    match args.language {
        Language::C | Language::Cpp => {
            eprintln!("Using C code generator");
            
            todo!("Create C generator")
        }
        Language::Rust => {
            eprintln!("Using Rust code generator");

            todo!("Create Rust generator")
        }
    }

    Ok(())
}
