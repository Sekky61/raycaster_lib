use config::Config;

mod args;
mod config;
mod file;
mod generators;
mod header;
mod sample_order;
mod shapes;

use crate::{args::get_command, generators::generate_vol};

pub fn main() {
    let cmd = get_command();
    // todo analyse header flag

    let m = cmd.get_matches();

    let cfg = Config::from(m);

    println!("Hello");
    println!("{:?}", cfg);

    generate_vol(cfg);
}
