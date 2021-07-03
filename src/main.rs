use anyhow::Context;
use dialoguer::MultiSelect;
use lexpr::Value;
use regex::Regex;
use sexpr::Text;
use std::{
    borrow::Cow,
    collections::{BTreeMap, HashMap},
    fs,
    hash::Hash,
    iter,
    path::PathBuf,
};

use argh::FromArgs;

use crate::sexpr::IntoText;

mod sexpr;

/// Synchronize your pinouts between firmware and electrical designs.
#[derive(FromArgs, Debug)]
struct Arguments {
    #[argh(positional)]
    pub netlist: PathBuf,
    #[argh(positional)]
    pub reference: Vec<String>,
}

fn main() -> anyhow::Result<()> {
    let args: Arguments = argh::from_env();

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

    println!("Parsing document {} {} by {}", title, rev, company);

    let nets = sexpr["nets"]
        .list_iter()
        .unwrap()
        .map(|net| {
            let net_name = Net::new(&net);

            net.list_iter()
                .unwrap()
                .skip(3)
                .map(move |node| (Node::new(&node), net_name.clone()))
        })
        .flatten()
        .collect::<HashMap<_, _>>();

    // dbg!(nets);

    let pins = sexpr["libparts"]
        .list_iter()
        .unwrap()
        .map(|libpart| {
            libpart["pins"].list_iter().map(|pins| {
                let pins = pins.map(Pin::new).collect::<Cow<_>>();

                LibraryPart::from_aliases(&libpart)
                    .chain(iter::once(LibraryPart::new(&libpart)))
                    .map(move |part| (part, pins.clone()))
            })
        })
        .filter_map(|x| x)
        .flatten()
        .collect::<HashMap<_, _>>();

    // dbg!(&pins);

    let components = sexpr["components"]
        .list_iter()
        .unwrap()
        .map(|v| {
            let component = Component::new(v);

            let pins = pins.get(&component.libpart).map(|pins| {
                pins.iter()
                    .map(|pin| {
                        let node = Node {
                            reference: component.reference.clone(),
                            pin: pin.num.clone(),
                        };

                        (
                            pin,
                            nets.get(&node)
                                .unwrap_or_else(|| panic!("no nets entry for {:?}", node)),
                        )
                    })
                    .collect::<Vec<_>>()
            });

            (v["ref"].text_join(), (component, pins))
        })
        .collect::<BTreeMap<_, _>>();

    // TODO: filter earlier?
    let width = components.keys().map(|x| x.len()).max().unwrap_or(0);

    let (references, items): (Vec<_>, Vec<_>) = components
        .iter()
        .map(|(reference, (component, _))| {
            (
                reference,
                format!("{:>width$}: {}", reference, component.value, width = width),
            )
        })
        .unzip();

    let choices = MultiSelect::new()
        .items(&items)
        .interact()?
        .into_iter()
        .map(|i| {
            let reference = references[i];

            (reference, &components[reference].1)
        })
        .collect::<Vec<_>>();

    for (reference, choice) in choices {
        println!("{}: {:?}", reference, choice);
    }
    // let results = args
    //     .reference
    //     .into_iter()
    //     .map(Cow::<str>::Owned)
    //     .map(|reference| (reference.clone(), components.get(&reference)))
    //     .collect::<HashMap<Cow<str>, _>>();

    // println!("{:#?}", results);

    Ok(())
}

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Node<'t> {
    pub reference: Text<'t>,
    pub pin: Text<'t>,
}

impl<'t> Node<'t> {
    pub fn new(node: &'t Value) -> Self {
        Node {
            reference: node["ref"].text_join(),
            pin: node["pin"].text_join(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Net<'t> {
    Custom(Text<'t>),
    Generated(Text<'t>),
}

impl<'t> Net<'t> {
    pub fn new(net: &'t Value) -> Self {
        let name = net["name"].text_join();

        if name.starts_with("Net-(") {
            Net::Generated(name)
        } else {
            Net::Custom(name)
        }
    }
}

#[derive(Debug, Clone)]
pub struct Component<'t> {
    libpart: LibraryPart<'t>,
    value: Text<'t>,
    description: Text<'t>,
    footprint: Text<'t>,
    datasheet: Text<'t>,
    reference: Text<'t>,
}

impl<'t> Component<'t> {
    pub fn new(value: &'t Value) -> Self {
        Self {
            libpart: LibraryPart::new(&value["libsource"]),
            description: value["libsource"]["description"].text_join(),
            value: value["value"].text_join(),
            footprint: value["footprint"].text_join(),
            datasheet: value["datasheet"].text_join(),
            reference: value["ref"].text_join(),
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

    pub fn from_aliases(value: &'t Value) -> impl Iterator<Item = LibraryPart<'t>> {
        value["aliases"]
            .list_iter()
            .map(|iter| {
                iter.map(move |alias| Self {
                    lib: value["lib"].text_join(),
                    part: alias[1].text_join(),
                })
            })
            .into_iter()
            .flatten()
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
