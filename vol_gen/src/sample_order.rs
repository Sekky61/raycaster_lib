use nalgebra::{vector, Vector3};

use crate::config::Config;

pub struct DimIterator<SRC>
where
    SRC: Iterator<Item = Vector3<u32>>,
{
    it: SRC,
}

impl<SRC> Iterator for DimIterator<SRC>
where
    SRC: Iterator<Item = Vector3<u32>>,
{
    type Item = Vector3<u32>;

    fn next(&mut self) -> Option<Self::Item> {
        self.it.next()
    }
}
