use nalgebra::{vector, Vector3};

#[derive(Debug)]
pub struct LinearCoordIterator {
    pub dims: Vector3<u32>,
    pub state: Vector3<u32>,
    done: bool,
    started: bool,
}

impl LinearCoordIterator {
    pub fn from_dims(dims: Vector3<u32>) -> LinearCoordIterator {
        LinearCoordIterator {
            dims,
            state: vector![0, 0, 0],
            done: false,
            started: false,
        }
    }

    pub fn reset(&mut self) {
        self.state = vector![0, 0, 0];
        self.done = false;
        self.started = false;
    }
}

impl Iterator for LinearCoordIterator {
    type Item = Vector3<u32>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        if !self.started {
            self.started = true;
            return Some(self.state);
        }
        self.state.z += 1;
        if self.state.z == self.dims.z {
            self.state.z = 0;
            self.state.y += 1;
        }
        if self.state.y == self.dims.y {
            self.state.y = 0;
            self.state.x += 1;
        }
        if self.state.x == self.dims.x {
            self.state.x = 0;
            self.done = true;
            return None;
        }

        Some(self.state)
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn lin_order() {
        let order = LinearCoordIterator::from_dims(vector![2, 3, 4]);

        #[rustfmt::skip]
        let expected = &[
            vector![0,0,0],vector![0,0,1],vector![0,0,2],vector![0,0,3],
            vector![0,1,0],vector![0,1,1],vector![0,1,2],vector![0,1,3],
            vector![0,2,0],vector![0,2,1],vector![0,2,2],vector![0,2,3],
            // next slice
            vector![1,0,0],vector![1,0,1],vector![1,0,2],vector![1,0,3],
            vector![1,1,0],vector![1,1,1],vector![1,1,2],vector![1,1,3],
            vector![1,2,0],vector![1,2,1],vector![1,2,2],vector![1,2,3],
        ];

        let order_collect: Vec<_> = order.collect();

        assert_eq!(order_collect.len(), 2 * 3 * 4);
        assert_eq!(order_collect.len(), expected.len());

        order_collect
            .iter()
            .zip(expected.iter())
            .for_each(|(act, exp)| assert_eq!(act, exp));
    }
}
