//! Generators module
//!
//! Types implmenting `SampleGenerator` can be used to generate volumes

use std::{error::Error, io::Write};

use nalgebra::Vector3;

use crate::{
    config::{Config, GeneratorConfig},
    file::open_create_file,
    header::generate_header,
    orders::{LinearCoordIterator, SampleOrder, ZCoordIterator},
};

use self::{shapes::ShapesGenerator, solid::SolidGenerator};

mod shapes;
mod solid;

// todo sparse files with writes using lseek

// Any generator
pub trait Generator {
    fn generate(self);
}

// Generates continuous chunks of samples in any order
pub trait ChunkGenerator {}

/// Generates one sample at a time, at any location
pub trait SampleGenerator {
    /// Generate sample
    /// Returns sample value
    ///
    /// # Arguments
    ///
    /// * `coords` - coordinates of the sample
    fn sample_at(&self, coords: Vector3<u32>) -> u8;
}

/// Obtain source of data
pub fn get_sample_generator(config: &Config) -> Box<dyn SampleGenerator> {
    match config.generator {
        GeneratorConfig::Shapes { .. } => Box::new(ShapesGenerator::from_config(config)),
        GeneratorConfig::Noise => todo!(),
        GeneratorConfig::Solid { .. } => Box::new(SolidGenerator::from_config(config)),
    }
}

/// Generate header and samples into file
/// Samples are in linear order
pub fn generate_linear_order(
    sg: Box<dyn SampleGenerator>,
    config: &Config,
) -> Result<(), Box<dyn Error>> {
    // Open file
    let file_name = &config.file_name;
    let mut file = open_create_file(file_name)?;

    // Write header
    let header = generate_header(config);
    let h_written = file.write(&header[..]).unwrap();
    if h_written != header.len() {
        return Err("Writing header error".into());
    }

    // Write samples in linear order
    let ord_iter = LinearCoordIterator::from_dims(config.dims);

    for dims in ord_iter {
        let sample = sg.sample_at(dims);
        let written = file.write(&[sample])?;

        if written != 1 {
            return Err("Writing error".into());
        }
    }

    Ok(())
}

/// Generate header and samples into file
/// Samples are in Z order
pub fn generate_z_order(
    sg: Box<dyn SampleGenerator>,
    config: &Config,
    block_side: u32,
) -> Result<(), Box<dyn Error>> {
    // Open file
    let file_name = &config.file_name;
    let mut file = open_create_file(file_name)?;

    // Write header
    let header = generate_header(config);
    let h_written = file.write(&header[..]).unwrap();
    if h_written != header.len() {
        return Err("Writing header error".into());
    }

    // Write samples
    let ord_iter = ZCoordIterator::new(config.dims, block_side);

    for dims in ord_iter {
        let sample = sg.sample_at(dims);
        let written = file.write(&[sample])?;

        if written != 1 {
            return Err("Writing error".into());
        }
    }

    Ok(())
}

pub fn generate_vol(config: Config) {
    let gen = get_sample_generator(&config);

    match config.save_buffer_order {
        SampleOrder::Linear => generate_linear_order(gen, &config).unwrap(),
        SampleOrder::Z(side) => generate_z_order(gen, &config, side as u32).unwrap(),
    }

    println!("Generating finished, result in {:#?}", config.file_name);
}
