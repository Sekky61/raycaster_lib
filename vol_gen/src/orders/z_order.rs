use nalgebra::{vector, Vector3};

use super::LinearCoordIterator;

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

// Could be also implemented with two nested LinearCoordIterators
#[derive(Debug)]
pub struct ZCoordIterator {
    // state
    block: LinearCoordIterator,
    inner: LinearCoordIterator,
    block_side: u32,
}

impl ZCoordIterator {
    pub fn new(dims: Vector3<u32>, block_side: u32) -> Self {
        assert_ne!(block_side, 0);
        let blocks = blockify(dims, block_side, 0); // todo overlap
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
        self.block.state * self.block_side + self.inner.state
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
    fn z_order() {
        let order = ZCoordIterator::new(vector![4, 4, 4], 2);

        println!("{:?}", order);

        #[rustfmt::skip]
        let expected = &[
            // first block
            vector![0,0,0],vector![0,0,1],vector![0,1,0],vector![0,1,1],
            vector![1,0,0],vector![1,0,1],vector![1,1,0],vector![1,1,1],
            // second block
            vector![0,0,2],vector![0,0,3],vector![0,1,2],vector![0,1,3],
            vector![1,0,2],vector![1,0,3],vector![1,1,2],vector![1,1,3],
            // third block
            vector![0,2,0],vector![0,2,1],vector![0,3,0],vector![0,3,1],
            vector![1,2,0],vector![1,2,1],vector![1,3,0],vector![1,3,1],
            // fourth block
            vector![0,2,2],vector![0,2,3],vector![0,3,2],vector![0,3,3],
            vector![1,2,2],vector![1,2,3],vector![1,3,2],vector![1,3,3],

            // other
            vector![2,0,0],vector![2,0,1],vector![2,1,0],vector![2,1,1],
            vector![3,0,0],vector![3,0,1],vector![3,1,0],vector![3,1,1],
            // 
            vector![2,0,2],vector![2,0,3],vector![2,1,2],vector![2,1,3],
            vector![3,0,2],vector![3,0,3],vector![3,1,2],vector![3,1,3],
            // 
            vector![2,2,0],vector![2,2,1],vector![2,3,0],vector![2,3,1],
            vector![3,2,0],vector![3,2,1],vector![3,3,0],vector![3,3,1],
            // 
            vector![2,2,2],vector![2,2,3],vector![2,3,2],vector![2,3,3],
            vector![3,2,2],vector![3,2,3],vector![3,3,2],vector![3,3,3],
        ];

        let order_collect: Vec<_> = order.collect();

        println!("{order_collect:?}");

        assert_eq!(order_collect.len(), 4 * 4 * 4);
        assert_eq!(order_collect.len(), expected.len());

        order_collect
            .iter()
            .zip(expected.iter())
            .for_each(|(act, exp)| assert_eq!(act, exp));
    }
}
