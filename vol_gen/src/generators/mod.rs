//! Generators module
//!
//! Types implmenting `SampleGenerator` can be used to generate volumes

use std::{error::Error, io::Write};

use indicatif::ProgressBar;
use nalgebra::Vector3;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{
    config::{Config, GeneratorConfig},
    file::open_create_file,
    generators::{shapes::ShapesGenerator, solid::SolidGenerator},
    header::generate_header,
    orders::{LinearCoordIterator, OrderGenerator, SampleOrder, ZCoordIterator},
};

mod shapes;
mod solid;

// todo sparse files with writes using lseek

/// Generates one sample at a time, at any location
pub trait SampleGenerator: Sync {
    /// Generate sample
    /// Returns sample value
    ///
    /// # Arguments
    ///
    /// * `coords` - coordinates of the sample
    fn sample_at(&self, coords: Vector3<u32>) -> u8;

    fn construct(config: &Config) -> Self;
}

/// Generate header and samples into file
/// Samples are in order dictated by generic parameter `OG`.
pub fn generate_order<SG: SampleGenerator, OG: OrderGenerator>(
    config: &Config,
) -> Result<(), Box<dyn Error>> {
    let sample_generator = SG::construct(config);
    let mut ord_iter = OG::construct(config);

    // ProgressBar
    let (_, total) = ord_iter.get_progress();
    let bar = ProgressBar::new(total);

    // Open file
    let file_name = &config.file_name;
    let mut file = open_create_file(file_name)?;

    // Write header
    let header = generate_header(config);
    let h_written = file.write(&header[..]).unwrap();
    if h_written != header.len() {
        return Err("Writing header error".into());
    }

    loop {
        // Get batch of coordinates
        let mut batch = vec![];

        for _ in 0..32768 {
            if let Some(pos) = ord_iter.next() {
                batch.push(pos);
            } else {
                break;
            }
        }
        if batch.is_empty() {
            break;
        }

        // Sample in parallel and collect results

        let output_samples: Vec<u8> = batch
            .par_iter()
            .map(|&pos| sample_generator.sample_at(pos))
            .collect();

        // Write into file
        let written = file.write(&output_samples)?;
        if written != output_samples.len() {
            return Err("Writing error".into());
        }

        let (current, _) = ord_iter.get_progress();
        bar.set_position(current);
    }

    Ok(())
}

pub fn generate_vol(config: Config) {
    let res: Result<(), Box<dyn Error>> = match (config.generator, config.save_buffer_order) {
        (GeneratorConfig::Shapes { .. }, SampleOrder::Linear) => {
            generate_order::<ShapesGenerator, LinearCoordIterator>(&config)
        }
        (GeneratorConfig::Shapes { .. }, SampleOrder::Z(_)) => {
            generate_order::<ShapesGenerator, ZCoordIterator>(&config)
        }
        (GeneratorConfig::Solid { .. }, SampleOrder::Linear) => {
            generate_order::<SolidGenerator, LinearCoordIterator>(&config)
        }
        (GeneratorConfig::Solid { .. }, SampleOrder::Z(_)) => {
            generate_order::<SolidGenerator, ZCoordIterator>(&config)
        }
        (GeneratorConfig::Noise, SampleOrder::Linear) => todo!(),
        (GeneratorConfig::Noise, SampleOrder::Z(_)) => todo!(),
    };

    res.unwrap();

    println!("Generating finished, result in {:#?}", config.file_name);
}
