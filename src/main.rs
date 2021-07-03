use anyhow::Context;
use netlist::Netlist;
use regex::Regex;
use std::fs;

use crate::args::Arguments;

mod args;
mod netlist;
mod sexpr;

fn main() -> anyhow::Result<()> {
    let args: Arguments = argh::from_env();

    let sexpr = {
        let netlist_file =
            fs::read_to_string(args.netlist).context("Failed to open the provided netlist file")?;

        let regex = Regex::new("(  )?\\(tstamp .*?\\)").unwrap();

        lexpr::from_str(&regex.replace_all(&netlist_file, ""))
            .context("Failed to parse the given netlist file")?
    };

    let netlist = Netlist::new(&sexpr);

    println!(
        "Parsing document {} {} by {}",
        netlist.title, netlist.rev, netlist.company
    );

    Ok(())
}
