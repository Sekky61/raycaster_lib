use nalgebra::Vector3;

use crate::config::{Config, GeneratorConfig};

use super::SampleGenerator;

pub struct SolidGenerator {
    sample: u8,
}

impl SolidGenerator {
    pub fn from_config(config: &Config) -> SolidGenerator {
        let sample = match config.generator {
            GeneratorConfig::Solid { sample } => sample,
            _ => panic!("Bad generator config"),
        };

        SolidGenerator { sample }
    }
}

impl SampleGenerator for SolidGenerator {
    fn sample_at(&self, _coords: Vector3<u32>) -> u8 {
        self.sample
    }
}
