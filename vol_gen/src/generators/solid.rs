/*
    vol_gen
    Author: Michal Majer
    Date: 2022-05-05
*/

use nalgebra::{vector, Vector3};

use crate::config::{Config, GeneratorConfig};

use super::SampleGenerator;

/// Generate solid volume
/// All sample values are the same
pub struct SolidGenerator {
    /// The sample value
    sample: u8,
    pad: u32,
    dims: Vector3<u32>,
}

impl SolidGenerator {
    pub fn from_config(config: &Config) -> SolidGenerator {
        let sample = match config.generator {
            GeneratorConfig::Solid { sample } => sample,
            _ => panic!("Bad generator config"),
        };

        SolidGenerator {
            sample,
            pad: 5,
            dims: config.dims,
        } // todo configurable
    }
}

impl SampleGenerator for SolidGenerator {
    fn sample_at(&self, coords: Vector3<u32>) -> u8 {
        let pad_end = self.dims - vector![self.pad, self.pad, self.pad];
        if coords.x < self.pad
            || coords.y < self.pad
            || coords.z < self.pad
            || coords.x > pad_end.x
            || coords.y > pad_end.y
            || coords.z > pad_end.z
        {
            0
        } else {
            self.sample
        }
    }

    fn construct(config: &Config) -> Self {
        SolidGenerator::from_config(config)
    }
}
