use anyhow::Context;
use regex::Regex;
use std::{fs, path::PathBuf};

use argh::FromArgs;

use crate::netlist::Netlist;

mod netlist;
mod sexpr;

/// Synchronize your pinouts between firmware and electrical designs.
#[derive(FromArgs, Debug)]
struct Arguments {
    #[argh(positional)]
    pub netlist: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args: Arguments = argh::from_env();
    dbg!(&args);

    let netlist_sexpr = {
        let netlist_file =
            fs::read_to_string(args.netlist).context("Failed to open the provided netlist file")?;

        let regex = Regex::new("(  )?\\(tstamp .*?\\)").unwrap();

        lexpr::from_str(&regex.replace_all(&netlist_file, ""))
            .context("Failed to parse the given netlist file")?
    };

    let netlist = Netlist::new(&netlist_sexpr);

    dbg!(&netlist);

    Ok(())
}
