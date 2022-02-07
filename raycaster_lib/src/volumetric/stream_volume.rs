use std::fs::File;

use memmap::{Mmap, MmapOptions};
use nalgebra::{vector, Vector3};

pub struct StreamVolume {
    size: Vector3<usize>,
    border: u32,
    scale: Vector3<f32>,    // shape of voxels
    vol_dims: Vector3<f32>, // size * scale = resulting size of bounding box ; max of bounding box
    file_map: Mmap,
}

impl std::fmt::Debug for StreamVolume {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StreamVolume")
            .field("size", &self.size)
            .field("border", &self.border)
            .field("scale", &self.scale)
            .field("vol_dims", &self.vol_dims)
            .field("file_map", &self.file_map)
            .finish()
    }
}

impl StreamVolume {
    pub fn new() -> Result<StreamVolume, std::io::Error> {
        let file = File::open("testfile")?;
        let mmap = unsafe { MmapOptions::new().map(&file)? };

        println!("{:?}", mmap);
        println!("{:?}", &mmap[..]);
        let vol = StreamVolume {
            size: vector![1, 1, 1],
            border: 0,
            scale: vector![1.0, 1.0, 1.0],
            vol_dims: vector![1.0, 1.0, 1.0],
            file_map: mmap,
        };
        Ok(vol)
    }
}

#[cfg(test)]
mod test {

    use nalgebra::{point, vector};

    use crate::ray::Ray;

    use super::*;

    #[test]
    fn t1() {
        println!("{:?}", std::env::current_dir());
        let vol = StreamVolume::new();
        println!("{:?}", vol);
    }
}
