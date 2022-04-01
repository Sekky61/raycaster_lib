use config::Config;

mod args;
mod config;
mod file;
mod generators;
mod header;
mod orders;

use crate::{args::get_command, generators::generate_vol};

pub fn main() {
    let cmd = get_command();
    // todo analyse header flag

    let args = cmd.get_matches();

    let cfg = Config::from_args(args);

    let cfg = match cfg {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: {e}");
            return;
        }
    };

    println!("Generating volume...");
    println!("{:?}", cfg);

    generate_vol(cfg);
}
