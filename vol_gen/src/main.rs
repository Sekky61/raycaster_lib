//! Volume generation tool
//!
//! # Command line arguments
//!
//! * `-d, --dims=<X>,<Y>,<Z>` - Dimensions of volume
//! * `-g, --generator <NAME>` - Type of generator [possible values: `shapes`, `noise`, `solid`]
//! * `-h, --help` - Print help information
//! * `-l, --layout <SHAPE>` - Layout of samples in memory [default: linear] [possible values: `linear`, `z`]
//! * `-o, --output-file <FILE>` - File name to output [default: `a.vol`]
//! * `-s, --shape=<X>,<Y>,<Z>` - Shape of cell [default: 1 1 1]

use config::Config;

mod args;
mod config;
mod file;
mod generators;
mod header;
mod orders;

use crate::{args::get_command, generators::generate_vol};

// todo check what if folder does not exist?

// For convenience:
// cargo run --release --bin vol_gen -- --dims=256,256,256 --generator solid --sample 112 --output-file volumes/solid_256_linear.vol
pub fn main() {
    // Get commands
    let cmd = get_command();
    // todo header analysis

    // Parse args and build configuration
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

    // Generate to file
    generate_vol(cfg);
}
