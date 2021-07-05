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

            // Generated {date} {time} by pinout
            // - Source: `{title}` {rev}
            // - Author: {company}
        ",
        title = sheet.title,
        company = sheet.company,
        rev = sheet.rev,
        date = Local::now().format("%Y-%m-%d"),
        time = Local::now().format("%H:%M:%S")
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

    // FIXME: ?
    writedoc!(
        w,
        "
            #pragma region PICO_BUILTIN

            /// Raspberry Pi Pico Internal Pin
            ///
            /// A GPIO tied to the PS (Power Save) pin on the internal SMPS to switch
            /// between PFM (Low Power, 0) and PWM (High Power, 1) modes.
            ///
            /// IO Type    : Output
            static const unsigned int PIN_PSU_PS = 23;

            /// Raspberry Pi Pico Internal Pin
            ///
            /// USB bus (V_BUS) voltage sense, digital high when present
            ///
            /// IO Type    : Input
            static const unsigned int PIN_V_BUS_SENSE = 24;

            /// Raspberry Pi Pico Internal Pin
            ///
            /// Pico onboard LED
            ///
            /// IO Type    : Output
            static const unsigned int PIN_LED = 25;

            /// Raspberry Pi Pico Internal Pin
            ///
            /// Pico onboard LED
            ///
            /// IO Type    : Input (ADC 3)
            static const unsigned int PIN_V_SYS = 29;

            /// Raspberry Pi Pico Internal ADC channel
            ///
            /// This is the ADC channel to select the pin PIN_V_BAT_SENSE
            ///
            /// @see PIN_V_SYS
            static const unsigned int ADC_V_SYS = 2;

            #pragma endregion PICO_BUILTIN
        "
    )?;

    // FIXME: refactor to do this earlier
    for (pin, net) in &component.pins {
        let net = match net {
            Net::Label(net) => &net[1..],
            Net::Custom(net) => &net,
            Net::Generated(_net) => {
                // TODO:
                // eprintln!(
                //     "Skipping generated net {net} on pin #{num}",
                //     net = net,
                //     num = pin.num
                // );
                continue;
            }
        };

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
                    ///
                    /// Pin Name   : {name}
                    /// Pin Number : {num}
                    /// IO Type    : {ty} {adc}
                    /// Net Label  : {net}
                ",
                name = pin.name,
                num = pin.num,
                ty = pin.ty,
                adc = match adc_num {
                    Some(num) => format!("(ADC {})", num),
                    None => "".to_string(),
                },
                net = net
            )?;
            let pin_const_name = format!("PIN_{}", net.to_ascii_uppercase());
            constant(w, &pin_const_name, gpio_num)?;
            if let Some(num) = adc_num {
                writedoc!(
                    w,
                    "
    
                        /// Raspberry Pi Pico ADC channel
                        ///
                        /// This is the ADC channel to select the pin {name}
                        ///
                        /// @see {name}
                    ",
                    name = pin_const_name
                )?;
                constant(w, &format!("ADC_{}", net.to_ascii_uppercase()), num)?;
            }
        } else {
            // TODO:
            // eprintln!(
            //     "Skipping non gpio pin #{num} {name}",
            //     num = pin.num,
            //     name = pin.name
            // );
        }
    }

    writeln!(
        w,
        "\n#pragma endregion {reference}_PINOUT",
        reference = component.reference
    )?;

    Ok(())
}
