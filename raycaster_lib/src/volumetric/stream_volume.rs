use std::fs::File;

use memmap::{Mmap, MmapOptions};
use nalgebra::{vector, Vector3};

use super::{BuildVolume, ParsedVolumeBuilder, Volume};

pub struct StreamVolume {
    position: Vector3<f32>,
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
            position: vector![0.0, 0.0, 0.0],
            size: vector![1, 1, 1],
            border: 0,
            scale: vector![1.0, 1.0, 1.0],
            vol_dims: vector![1.0, 1.0, 1.0],
            file_map: mmap,
        };
        Ok(vol)
    }

    fn get_3d_index(&self, x: usize, y: usize, z: usize) -> usize {
        z + y * self.size.z + x * self.size.y * self.size.z
    }

    fn get_3d_data(&self, x: usize, y: usize, z: usize) -> f32 {
        //println!("Getting {} {} {}", x, y, z);
        let index = self.get_3d_index(x, y, z);
        let buf: &[u8] = self.file_map.as_ref();
        buf[28 + index] as f32
    }

    fn get_block_data_half(&self, base: usize) -> [f32; 4] {
        let buf: &[u8] = self.file_map.as_ref();
        [
            buf[base] as f32,
            buf[base + 1] as f32,
            buf[base + self.size.y] as f32,
            buf[base + self.size.y + 1] as f32,
        ]
    }
}

impl BuildVolume<ParsedVolumeBuilder<u8>> for StreamVolume {
    fn build(builder: ParsedVolumeBuilder<u8>) -> StreamVolume {
        println!("Build started");

        let data = if let Some(mmap) = builder.mmap {
            mmap
        } else {
            // todo error
            panic!("No file map in builder");
        };

        let vol_dims = (builder.size - vector![1, 1, 1]) // side length is n-1 times the point
            .cast::<f32>()
            .component_mul(&builder.scale);
        StreamVolume {
            position: vector![0.0, 0.0, 0.0],
            size: builder.size,
            border: 0,
            scale: builder.scale,
            vol_dims,
            file_map: data,
        }
    }
}

impl Volume for StreamVolume {
    fn get_size(&self) -> Vector3<usize> {
        self.size
    }

    fn get_pos(&self) -> Vector3<f32> {
        self.position
    }

    fn get_dims(&self) -> Vector3<f32> {
        self.vol_dims
    }

    fn sample_at(&self, pos: nalgebra::Point3<f32>) -> f32 {
        // todo taky zkusit rozseknout
        let x_low = pos.x as usize;
        let y_low = pos.y as usize;
        let z_low = pos.z as usize;

        let x_t = pos.x.fract();
        let y_t = pos.y.fract();
        let z_t = pos.z.fract();

        let base = self.get_3d_index(x_low, y_low, z_low) + 28;

        let first_index = base;
        let second_index = base + self.size.z * self.size.y;

        // first plane
        let first_data = self.get_block_data_half(first_index);
        let [c000, c001, c010, c011] = first_data;

        let inv_z_t = 1.0 - z_t;
        let inv_y_t = 1.0 - y_t;

        let c00 = c000 * inv_z_t + c001 * z_t; // z low
        let c01 = c010 * inv_z_t + c011 * z_t; // z high
        let c0 = c00 * inv_y_t + c01 * y_t; // point on yz plane

        // second plane
        let second_data = self.get_block_data_half(second_index);
        let [c100, c101, c110, c111] = second_data;

        let c10 = c100 * inv_z_t + c101 * z_t; // z low
        let c11 = c110 * inv_z_t + c111 * z_t; // z high
        let c1 = c10 * inv_y_t + c11 * y_t; // point on yz plane

        c0 * (1.0 - x_t) + c1 * x_t
    }

    fn get_data(&self, x: usize, y: usize, z: usize) -> f32 {
        self.get_3d_data(x, y, z)
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
