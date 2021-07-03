use std::{borrow::Cow, collections::HashMap, iter, ops::Deref};
use strum::Display;

use lexpr::Value;

use crate::sexpr::{IntoText, Text};

#[derive(Debug)]
pub struct Netlist<'t> {
    pub sheet: Sheet<'t>,
    pub components: HashMap<String, AssociatedComponent<'t>>,
}

#[derive(Debug)]
pub struct AssociatedComponent<'t> {
    component: Component<'t>,
    pub pins: Vec<(Pin<'t>, Net<'t>)>,
}

impl<'t> Deref for AssociatedComponent<'t> {
    type Target = Component<'t>;

    fn deref(&self) -> &Self::Target {
        &self.component
    }
}

#[derive(Debug)]
pub struct Sheet<'t> {
    pub title: Text<'t>,
    pub company: Text<'t>,
    pub rev: Text<'t>,
}

impl<'t> Netlist<'t> {
    pub fn new(sexpr: &'t Value) -> Self {
        let sheet = {
            let title_block = &sexpr["design"]["sheet"]["title_block"];

            Sheet {
                title: title_block["title"].text_join(),
                company: title_block["company"].text_join(),
                rev: title_block["rev"].text_join(),
            }
        };

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

        let components = sexpr["components"]
            .list_iter()
            .unwrap()
            .map(|v| {
                let component = Component::new(v);

                let pins = pins
                    .get(&component.libpart)
                    .cloned()
                    .map(|pins| {
                        pins.into_iter()
                            .map(|pin| {
                                let node = Node {
                                    reference: component.reference.clone(),
                                    pin: pin.num.clone(),
                                };

                                (
                                    pin.clone(),
                                    nets.get(&node)
                                        .cloned()
                                        .unwrap_or_else(|| panic!("no nets entry for {:?}", node)),
                                )
                            })
                            .collect::<Vec<_>>()
                    })
                    .into_iter()
                    .flatten()
                    .collect::<Vec<_>>();

                (
                    v["ref"].text_join().to_uppercase(),
                    AssociatedComponent { component, pins },
                )
            })
            .collect::<HashMap<_, _>>();

        Netlist { sheet, components }
    }
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
    Label(Text<'t>),
}

impl<'t> Net<'t> {
    pub fn new(net: &'t Value) -> Self {
        let name = net["name"].text_join();

        if name.starts_with("Net-(") {
            Net::Generated(name)
        } else if name.starts_with("/") {
            Net::Label(name)
        } else {
            Net::Custom(name)
        }
    }
}

#[derive(Debug, Clone)]
pub struct Component<'t> {
    pub libpart: LibraryPart<'t>,
    pub value: Text<'t>,
    pub description: Text<'t>,
    pub footprint: Text<'t>,
    pub datasheet: Text<'t>,
    pub reference: Text<'t>,
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
    pub lib: Text<'t>,
    pub part: Text<'t>,
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
pub struct Pin<'t> {
    pub num: Text<'t>,
    pub name: Text<'t>,
    pub ty: PinType,
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

#[derive(Debug, Clone, Copy, Display)]
pub enum PinType {
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
