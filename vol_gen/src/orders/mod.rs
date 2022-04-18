//! Orders in which samples can be laid out

mod linear_order;
mod z_order;

pub use linear_order::LinearCoordIterator;
use nalgebra::Vector3;
pub use z_order::ZCoordIterator;

use crate::config::Config;

// Order of samples in resulting array
// Generally, the fastest growing axis is Z axis
#[derive(Debug, Clone, Copy)]
pub enum SampleOrder {
    // Samples ordered by axis (x,y,z)
    Linear,

    // Samples ordered by blocks
    // Blocks are ordered lineary
    // and data inside blocks is also ordered lineary
    Z(u8),
}

pub trait OrderGenerator: Iterator<Item = Vector3<u32>> {
    fn construct(config: &Config) -> Self;
}
