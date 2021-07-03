use anyhow::Context;
use chrono::Local;
use indoc::writedoc;
use regex::Regex;
use std::io::{self, ErrorKind, Write};

use crate::netlist::{AssociatedComponent, Net, Sheet};

pub fn generate<'t>(
    w: &mut dyn Write,
    sheet: &Sheet<'t>,
    component: &AssociatedComponent<'t>,
) -> anyhow::Result<()> {
    fn constant(w: &mut dyn Write, name: &str, value: u64) -> io::Result<()> {
        writeln!(
            w,
            "static const unsigned int {name} = {value};",
            name = name,
            value = value
        )
    }

    writedoc!(
        w,
        "
            #pragma once

            // Generated {date} by pinout
            // - Source: `{title}` {rev}
            // - Author: {company}
        ",
        title = sheet.title,
        company = sheet.company,
        rev = sheet.rev,
        date = Local::now().format("%Y-%m-%d")
    )?;

    writedoc!(
        w,
        "


            // ----------Begin component `{reference}`---------
            // - Library: {lib}
            // - Part: {part}
            // - Value: {value}
            // - Description: {description}
            // - Footprint: {footprint}
            // - Datasheet: {datasheet}
            #pragma region {reference}_PINOUT
        ",
        reference = component.reference,
        lib = component.libpart.lib,
        part = component.libpart.part,
        value = component.value,
        description = component.description,
        footprint = component.footprint,
        datasheet = component.datasheet,
    )?;

    let gpio_regex = Regex::new("GPIO([0-9]+)(?:_ADC([0-9]+))?")
        .map_err(|e| io::Error::new(ErrorKind::InvalidData, e))?;

    for (pin, net) in &component.pins {
        let net = match net {
            Net::Label(net) => &net[1..],
            Net::Custom(net) => &net,
            Net::Generated(net) => {
                writeln!(
                    w,
                    "\n// Skipping generated net {net} on pin #{num}",
                    net = net,
                    num = pin.num
                )?;
                continue;
            }
        };

        // TODO: refactor to do this earlier

        if let Some(captures) = gpio_regex.captures(&pin.name) {
            let gpio_num = captures[1].parse().context("Invalid GPIO number")?;
            let adc_num = captures
                .get(2)
                .map(|x| x.as_str().parse::<u64>().context("Invalid ADC number"))
                .transpose()?;

            writedoc!(
                w,
                "

                    /// Raspberry Pi Pico GPIO Pin
                    /// Name: {name}
                    /// Number: {num}
                    /// Type: {ty}
                ",
                name = pin.name,
                num = pin.num,
                ty = pin.ty
            )?;
            writeln!(
                w,
                "/// ADC Input: {}",
                match adc_num {
                    Some(num) => num.to_string(),
                    None => "NO".to_string(),
                }
            )?;
            constant(w, net, gpio_num)?;
        } else {
            writeln!(
                w,
                "\n// Skipping non gpio pin #{num} {name}",
                num = pin.num,
                name = pin.name
            )?;
        }
    }

    writeln!(
        w,
        "#pragma endregion {reference}_PINOUT",
        reference = component.reference
    )?;

    Ok(())
}
