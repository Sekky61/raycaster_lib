use std::ffi::OsStr;

use clap::{Arg, Command, ValueHint};

// up to 32bit value
pub fn is_positive_number(num: &str) -> Result<(), String> {
    let n = num.parse::<u32>();
    match n {
        Ok(n) => {
            if n > 0 {
                Ok(())
            } else {
                Err("Number must be greater than 0".into())
            }
        }
        Err(_) => Err("Number required".into()),
    }
}

pub fn can_fit_u8(num: &str) -> Result<(), String> {
    let n = num.parse::<u8>();
    match n {
        Ok(_) => Ok(()),
        Err(_) => Err("Number does not fit in range <0;255>".into()),
    }
}

pub fn is_float_number(num: &str) -> Result<(), String> {
    let n = num.parse::<f32>();
    match n {
        Ok(n) => {
            if n > 0.0 {
                Ok(())
            } else {
                Err("Number must be greater than 0.0".into())
            }
        }
        Err(_) => Err("Number required".into()),
    }
}

const GENERATOR_NAMES: &[&str] = &["shapes", "noise", "solid"];
const LAYOUT_NAMES: &[&str] = &["linear", "z"];

pub fn get_command<'a>() -> Command<'a> {
    Command::new("My Program")
        .author("Michal Majer")
        .version("0.1.0")
        .about("Volumetric data generator")
        .arg(
            Arg::new("dims")
                .help("Dimensions of volume")
                .long("dims")
                .short('d')
                .required(true)
                .number_of_values(3)
                .value_names(&["X", "Y", "Z"])
                .use_value_delimiter(true)
                .require_value_delimiter(true)
                .require_equals(true)
                .validator(is_positive_number),
        )
        .arg(
            Arg::new("shape")
                .help("Shape of cell")
                .long("shape")
                .short('s')
                .number_of_values(3)
                .value_names(&["X", "Y", "Z"])
                .use_value_delimiter(true)
                .require_value_delimiter(true)
                .require_equals(true)
                .default_values(&["1", "1", "1"])
                .validator(is_float_number),
        )
        .arg(
            Arg::new("generator")
                .help("Type of generator")
                .long("generator")
                .short('g')
                .required(true)
                .requires_ifs(&[
                    ("solid", "sample"), // if solid is set, require sample
                ])
                .takes_value(true)
                .value_name("NAME")
                .possible_values(GENERATOR_NAMES),
        )
        .arg(
            Arg::new("layout")
                .help("Layout of samples in memory")
                .long("layout")
                .short('l')
                .default_value("linear")
                .value_name("SHAPE")
                .possible_values(LAYOUT_NAMES),
        )
        .arg(
            Arg::new("block-size") // maybe join this with layout arg | todo add overlap default 1
                .help("Size of blocks in Z shape layout")
                .long("block-size")
                .short('b')
                .value_name("SIDE")
                .hide(true) // Hide from help
                .required_if_eq("layout", "z")
                .validator(|s| is_positive_number(s).and(can_fit_u8(s))),
        )
        .arg(
            Arg::new("output-file")
                .help("File name to output")
                .long("output-file")
                .short('o')
                .value_name("FILE")
                .allow_invalid_utf8(true)
                .value_hint(ValueHint::FilePath)
                .default_value_os(OsStr::new("a.vol")),
        )
        .arg(Arg::new("sparse").help("Use sparse files").long("sparse"))
}
