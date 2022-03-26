use nalgebra::Vector3;

use crate::{config::Config, header};

pub struct SolidGenerator {
    sample: u8,
    dims: Vector3<usize>,
}

impl SolidGenerator {
    #[must_use]
    pub fn new(sample: u8, dims: Vector3<usize>) -> Self {
        Self { sample, dims }
    }

    pub fn from_config(cfg: Config, sample: u8) -> SolidGenerator {
        let dims = cfg.dims;
        let header = header::generate_header(cfg);

        todo!()
    }
}
