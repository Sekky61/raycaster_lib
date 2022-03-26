use std::{error::Error, io::Write};

use nalgebra::Vector3;

use crate::{config::Config, file::open_create_file, sample_order::DimIterator};

mod solid;

// Generates continuous chunks of samples in any order
// Writes using lseek
pub trait ChunkGenerator {}

// Generates one sample at a time, at random location
pub trait SampleGenerator {
    fn generate(&self) -> Result<(), Box<dyn Error>>;

    fn sample_at(&self, coords: Vector3<usize>) -> u8;

    fn generate_with_order<O>(&self, mut ord_iter: O) -> Result<(), Box<dyn Error>>
    where
        O: DimIterator + Iterator<Item = Vector3<usize>>,
    {
        let mut file = open_create_file("foo.vol")?;
        while let Some(dims) = Iterator::next(&mut ord_iter) {
            let sample = self.sample_at(dims);
            let written = file.write(&[sample]).unwrap();
        }

        Ok(())
    }
}
