use nalgebra::{vector, Vector3};

use crate::config::Config;

use super::{LinearCoordIterator, OrderGenerator, SampleOrder};

// Rounds up
// Tested in raycaster_lib
pub fn blockify(size: Vector3<u32>, side: u32, overlap: u32) -> Vector3<u32> {
    let cells = side - overlap; // cells per block
                                // (size-1) -- number of cells
                                // (+cells-1 / cells) -- division with rounding up
    let x = size + vector![cells, cells, cells];
    let y = x - vector![2, 2, 2];
    y / cells
}

// Implemented with two nested LinearCoordIterators
#[derive(Debug)]
pub struct ZCoordIterator {
    // state
    block: LinearCoordIterator,
    inner: LinearCoordIterator,
    block_side: u32,
}

impl OrderGenerator for ZCoordIterator {
    fn construct(config: &Config) -> Self {
        let block_side = match config.save_buffer_order {
            SampleOrder::Linear => {
                panic!("Constructing ZCoordIterator while setting SampleOrder to linear")
            }
            SampleOrder::Z(e) => e as u32,
        };
        ZCoordIterator::new(config.dims, block_side)
    }
}

impl ZCoordIterator {
    // block_side - number of voxels in one side of a block
    pub fn new(dims: Vector3<u32>, block_side: u32) -> Self {
        assert_ne!(block_side, 0);
        let blocks = blockify(dims, block_side, 1); // todo overlap
        let mut block = LinearCoordIterator::from_dims(blocks);
        block.next(); // Take first block implicitly

        let inner = LinearCoordIterator::from_dims(vector![block_side, block_side, block_side]);
        Self {
            block,
            inner,
            block_side,
        }
    }

    fn combine(&self) -> Vector3<u32> {
        // combine with overlap
        self.block.state * (self.block_side - 1) + self.inner.state
    }
}

impl Iterator for ZCoordIterator {
    type Item = Vector3<u32>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.inner.next() {
            Some(_) => Some(self.combine()),
            None => self.block.next().map(|_| {
                self.inner.reset();
                self.inner.next();
                self.combine()
            }),
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn z_order_evenly() {
        // should be 2x2x2 blocks
        let order = ZCoordIterator::new(vector![5, 5, 5], 3);

        println!("{:?}", order);

        #[rustfmt::skip]
        let expected = &[
            // first block
            vector![0,0,0],vector![0,0,1],vector![0,0,2],
            vector![0,1,0],vector![0,1,1],vector![0,1,2],
            vector![0,2,0],vector![0,2,1],vector![0,2,2],

            vector![1,0,0],vector![1,0,1],vector![1,0,2],
            vector![1,1,0],vector![1,1,1],vector![1,1,2],
            vector![1,2,0],vector![1,2,1],vector![1,2,2],
            
            vector![2,0,0],vector![2,0,1],vector![2,0,2],
            vector![2,1,0],vector![2,1,1],vector![2,1,2],
            vector![2,2,0],vector![2,2,1],vector![2,2,2],

            // second block (overlaps first)
            vector![0,0,2],vector![0,0,3],vector![0,0,4],
            vector![0,1,2],vector![0,1,3],vector![0,1,4],
            vector![0,2,2],vector![0,2,3],vector![0,2,4],

            vector![1,0,2],vector![1,0,3],vector![1,0,4],
            vector![1,1,2],vector![1,1,3],vector![1,1,4],
            vector![1,2,2],vector![1,2,3],vector![1,2,4],

            vector![2,0,2],vector![2,0,3],vector![2,0,4],
            vector![2,1,2],vector![2,1,3],vector![2,1,4],
            vector![2,2,2],vector![2,2,3],vector![2,2,4],

            // third block
            vector![0,2,0],vector![0,2,1],vector![0,2,2],
            vector![0,3,0],vector![0,3,1],vector![0,3,2],
            vector![0,4,0],vector![0,4,1],vector![0,4,2],

            vector![1,2,0],vector![1,2,1],vector![1,2,2],
            vector![1,3,0],vector![1,3,1],vector![1,3,2],
            vector![1,4,0],vector![1,4,1],vector![1,4,2],
            
            vector![2,2,0],vector![2,2,1],vector![2,2,2],
            vector![2,3,0],vector![2,3,1],vector![2,3,2],
            vector![2,4,0],vector![2,4,1],vector![2,4,2],
        ];

        let order_collect: Vec<_> = order.collect();

        // Has correct length
        assert_eq!(order_collect.len(), (2 * 2 * 2) * (3 * 3 * 3)); // blocks * elements in block

        // Zip has the length of shorter iterator, so only beginning is tested
        order_collect
            .iter()
            .zip(expected.iter())
            .enumerate()
            .for_each(|(i, (act, exp))| assert_eq!(act, exp, "comparison index [{i}] failed"));
    }

    // todo z-order padding test
}
