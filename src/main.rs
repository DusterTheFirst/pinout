use anyhow::Context;
use lexpr::Value;
use regex::Regex;
use sexpr::Text;
use std::{collections::HashMap, fs, hash::Hash, path::PathBuf};

use argh::FromArgs;

use crate::sexpr::IntoText;

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

    let sexpr = {
        let netlist_file =
            fs::read_to_string(args.netlist).context("Failed to open the provided netlist file")?;

        let regex = Regex::new("(  )?\\(tstamp .*?\\)").unwrap();

        lexpr::from_str(&regex.replace_all(&netlist_file, ""))
            .context("Failed to parse the given netlist file")?
    };

    let (title, company, rev) = {
        let title_block = &sexpr["design"]["sheet"]["title_block"];

        (
            title_block["title"].text_join(),
            title_block["company"].text_join(),
            title_block["rev"].text_join(),
        )
    };

    let nets = sexpr["nets"]
        .list_iter()
        .unwrap()
        .map(|net| {
            let name = net["name"].text_join();

            net.list_iter().unwrap().skip(2).map(move |node| {
                let reference = node["ref"].text_join();
                let pin = node["pin"].text_join();

                ((reference, pin), name.clone())
            })
        })
        .flatten()
        .collect::<HashMap<_, _>>();

    dbg!(nets);

    let pins = sexpr["libparts"]
        .list_iter()
        .unwrap()
        .map(|v| {
            v["pins"]
                .list_iter()
                .map(|pins| (LibraryPart::new(&v), pins.map(Pin::new).collect::<Vec<_>>()))
        })
        .filter_map(|a| a)
        .collect::<HashMap<_, _>>();

    // dbg!(pins);

    let components = sexpr["components"]
        .list_iter()
        .unwrap()
        .map(|v| {
            let component = Component::new(v);
            let pins = pins.get(&component.libpart);

            (v["ref"].text_join(), (component, pins))
        })
        .collect::<HashMap<_, _>>();

    // dbg!(components);

    Ok(())
}

#[derive(Debug, Clone)]
pub struct Component<'t> {
    libpart: LibraryPart<'t>,
    value: Text<'t>,
    description: Text<'t>,
    footprint: Text<'t>,
    datasheet: Text<'t>,
}

impl<'t> Component<'t> {
    pub fn new(value: &'t Value) -> Self {
        Self {
            libpart: LibraryPart::new(&value["libsource"]),
            description: value["libsource"]["description"].text_join(),
            value: value["value"].text_join(),
            footprint: value["footprint"].text_join(),
            datasheet: value["datasheet"].text_join(),
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct LibraryPart<'t> {
    lib: Text<'t>,
    part: Text<'t>,
}

impl<'t> LibraryPart<'t> {
    pub fn new(value: &'t Value) -> Self {
        Self {
            lib: value["lib"].text_join(),
            part: value["part"].text_join(),
        }
    }
}

#[derive(Debug, Clone)]
struct Pin<'t> {
    num: Text<'t>,
    name: Text<'t>,
    ty: PinType,
    // net: Net
}

impl<'t> Pin<'t> {
    pub fn new(value: &'t Value) -> Self {
        Self {
            num: value["num"].text_join(),
            name: value["name"].text_join(),
            ty: PinType::from(&value["type"].text_join()),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum PinType {
    Input,
    Output,
    BiDirectional,
    PowerIn,
    PowerOut,
    Passive,
    NotConnected,
    TriState,
    // Unspecified, TODO:
    // OpenCollector, TODO:
    // OpenEmitter TODO:
}

impl<S: AsRef<str>> From<S> for PinType {
    fn from(str: S) -> Self {
        let str = str.as_ref();

        match str.to_lowercase().as_str() {
            "passive" => PinType::Passive,
            "input" => PinType::Input,
            "output" => PinType::Output,
            "bidi" => PinType::BiDirectional,
            "power_in" => PinType::PowerIn,
            "power_out" => PinType::PowerOut,
            "notconnected" => PinType::NotConnected,
            "3state" => PinType::TriState,
            _ => unimplemented!("The pin type {} has not yet been implemented", str),
        }
    }
}
