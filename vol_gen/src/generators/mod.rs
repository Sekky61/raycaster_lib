use std::{error::Error, io::Write};

use nalgebra::Vector3;

use crate::{
    config::{Config, GeneratorConfig},
    file::open_create_file,
    header::generate_header,
    sample_order::LinearCoordIterator,
};

mod solid;

// todo sparse
// Writes using lseek

// Any generator
pub trait Generator {
    fn generate(self);
}

// Generates continuous chunks of samples in any order
pub trait ChunkGenerator {}

// Generates one sample at a time, at any location
pub trait SampleGenerator {
    fn sample_at(&self, coords: Vector3<u32>) -> u8;
}

pub fn get_sample_generator(config: &Config) -> impl SampleGenerator {
    match config.generator {
        GeneratorConfig::Shapes => todo!(),
        GeneratorConfig::Noise => todo!(),
        GeneratorConfig::Solid { .. } => solid::SolidGenerator::from_config(config),
    }
}

pub fn generate_linear_order(
    sg: impl SampleGenerator,
    config: &Config,
) -> Result<(), Box<dyn Error>> {
    let file_name = &config.file_name;
    let mut file = open_create_file(file_name)?;
    let ord_iter = LinearCoordIterator::from_config(config);

    let header = generate_header(config);
    let h_written = file.write(&header[..]).unwrap();
    if h_written != header.len() {
        return Err("Writing header error".into());
    }

    for dims in ord_iter {
        let sample = sg.sample_at(dims);
        let written = file.write(&[sample])?;

        if written != 1 {
            return Err("Writing error".into());
        }
    }

    Ok(())
}

// fn generate_with_order(&self) -> Result<(), Box<dyn Error>> {
//     let file_name = self.get_file_name();
//     let mut file = open_create_file(file_name)?;
//     let mut ord_iter = self.get_order_iter();

//     while let Some(dims) = Iterator::next(&mut ord_iter) {
//         let sample = self.sample_at(dims);
//         let written = file.write(&[sample]).unwrap();
//     }

//     Ok(())
// }
