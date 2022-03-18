use std::{error::Error, process::exit};

pub struct Config {
    dims: (usize, usize, usize),
    generator: String,
}

impl Config {
    pub fn new(dims: (usize, usize, usize), generator: String) -> Self {
        Self { dims, generator }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ConfigParseError;

impl Error for ConfigParseError {}

impl std::fmt::Display for ConfigParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "wrong arguments")
    }
}

//  Expected args
//
//  0       1   2   3   4
//  exe     x   y   z   generator
//
//  Example 100 100 100 noise
pub fn parse_args(args: &[String]) -> Result<Config, ConfigParseError> {
    if args.len() != 5 {
        return Err(ConfigParseError);
    }

    let x: usize = args[0].parse().map_err(|_| ConfigParseError)?;
    let y: usize = args[1].parse().map_err(|_| ConfigParseError)?;
    let z: usize = args[2].parse().map_err(|_| ConfigParseError)?;
    let generator = args[3].clone();

    match x == 0 {
        true => (),
        false => return Err(ConfigParseError),
    }

    Ok(Config::new((x, y, z), generator))
}

pub fn main() {
    let args: Vec<String> = std::env::args().collect();
    println!("Hello");
    println!("{args:?}");

    let config = match parse_args(&args[..]) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{e}");
            exit(1);
        }
    };

    generate(config);
}

fn generate(config: Config) {
    todo!()
}
