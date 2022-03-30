use nalgebra::Vector3;

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
