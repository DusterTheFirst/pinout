use lexpr::Value;

use crate::sexpr::{IntoText, Text};

#[derive(Debug)]
pub struct Netlist<'n> {
    pub sheet: Sheet<'n>,
}

#[derive(Debug)]
pub struct Sheet<'a> {
    pub title: Text<'a>,
    pub company: Text<'a>,
    pub rev: Text<'a>,
}

impl<'n> Netlist<'n> {
    pub fn new(v: &'n Value, component: &dyn AsRef<str>) -> Self {
        let (title, company, rev) = {
            let title_block = &v["design"]["sheet"]["title_block"];

            (
                title_block["title"][0].text(),
                title_block["company"][0].text(),
                title_block["rev"][0].text(),
            )
        };

        let components = v["components"]
            .list_iter()
            .expect("components was not a list");

        let nets = v["nets"]
            .list_iter()
            .expect("nets was not a list")
            .map(|net| net[code]);

        dbg!(components);

        Self {
            sheet: Sheet {
                title,
                company,
                rev,
            },
        }
    }
}
