use nalgebra::Vector3;

// Describe header
#[derive(Debug)]
pub enum HeaderFormat {
    Default,
}

// Order of samples in resulting array
// Generally, the fastest growing axis is Z axis
#[derive(Debug)]
pub enum SampleOrder {
    // Samples ordered by axis (x,y,z)
    Linear,

    // Samples ordered by blocks
    // Blocks are ordered lineary
    // and data inside blocks is also ordered lineary
    Z(u8), // todo parametrize overlap
}

pub trait DimIterator {}

impl<T> DimIterator for T where T: Iterator<Item = Vector3<usize>> {}

pub struct LinearCoordIterator {
    dims: Vector3<usize>,
    state: Vector3<usize>,
}

impl Iterator for LinearCoordIterator {
    type Item = Vector3<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

// Could be also implemented with two nested LinearCoordIterators
pub struct ZCoordIterator {
    // state
    current_block: Vector3<usize>,
    current_offset: usize,
    // dims
    block_side: usize,
    dims_block: Vector3<usize>,
}

impl Iterator for ZCoordIterator {
    type Item = Vector3<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}
