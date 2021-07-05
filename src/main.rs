use anyhow::Context;
use netlist::Netlist;
use regex::Regex;
use std::fs::{self, File};

use crate::{args::Arguments, codegen::get_generator};

mod args;
mod codegen;
mod netlist;
mod sexpr;

fn main() -> anyhow::Result<()> {
    let args: Arguments = argh::from_env();

    eprintln!("Loading netlist {:?}", args.netlist);

    let sexpr = {
        let netlist_file = fs::read_to_string(&args.netlist)
            .context("Failed to open the provided netlist file")?;

        let regex = Regex::new("(  )?\\(tstamp .*?\\)").unwrap();

        lexpr::from_str(&regex.replace_all(&netlist_file, ""))
            .context("Failed to parse the given netlist file")?
    };

    let Netlist { sheet, components } = Netlist::new(&sexpr);

    eprintln!(
        "Found sheet `{}` {} by {}",
        sheet.title, sheet.rev, sheet.company,
    );

    let component = match components.get(&args.reference.to_uppercase()) {
        Some(c) => c,
        None => {
            eprintln!(
                "Could not find component with ref {ref}",
                r#ref = args.reference
            );

            return Ok(());
        }
    };

    eprintln!(
        "Found component with ref {ref}: {value}",
        r#ref = args.reference,
        value = component.value
    );

    eprintln!(
        "Generating {} file `{}`",
        args.language,
        args.output_file.to_string_lossy(),
    );

    let mut file = File::create(args.output_file).context("Could not create output file")?;

    let generator = get_generator(args.language);

    generator(&mut file, &sheet, component).context("Failed to create code")?;

    eprintln!("Done");

    Ok(())
}
