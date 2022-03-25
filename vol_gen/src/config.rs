use std::ffi::{OsStr, OsString};

use clap::ArgMatches;
use nalgebra::Vector3;

use crate::sample_order::{HeaderFormat, SampleOrder};

// todo Describe command args here

#[derive(Debug)]
pub struct Config {
    dims: Vector3<usize>,
    cell_shape: Vector3<f32>,
    generator: GeneratorConfig,
    header_format: HeaderFormat,
    save_buffer_order: SampleOrder,
    file_name: OsString,
    sparse_file: bool,
}

impl From<ArgMatches> for Config {
    fn from(a: ArgMatches) -> Self {
        todo!()
    }
}

#[derive(Debug)]
pub struct GeneratorConfig {
    gen_type: GeneratorType,
    args: String,
}

#[derive(Debug)]
pub enum GeneratorType {
    Shapes,
    Noise,
}
