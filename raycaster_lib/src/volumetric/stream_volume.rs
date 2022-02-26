use memmap::Mmap;
use nalgebra::{vector, Point3, Vector3};

use crate::volumetric::vol_builder::DataSource;

use super::{vol_builder::VolumeMetadata, BuildVolume, Volume, TF};

pub struct StreamVolume {
    position: Vector3<f32>,
    size: Vector3<usize>,
    border: u32,
    scale: Vector3<f32>,    // shape of voxels
    vol_dims: Vector3<f32>, // size * scale = resulting size of bounding box ; max of bounding box
    file_map: Mmap,
    map_offset: usize,
    tf: TF,
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
    fn get_3d_index(&self, x: usize, y: usize, z: usize) -> usize {
        z + y * self.size.z + x * self.size.y * self.size.z
    }

    fn get_3d_data(&self, x: usize, y: usize, z: usize) -> f32 {
        //println!("Getting {} {} {}", x, y, z);
        let index = self.map_offset + self.get_3d_index(x, y, z);
        let buf: &[u8] = self.file_map.as_ref();
        buf[index] as f32
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

impl BuildVolume<u8> for StreamVolume {
    fn build(metadata: VolumeMetadata<u8>>) -> Result<StreamVolume, &'static str> {
        println!("Build started");

        let data = metadata.data.ok_or("No data")?;

        let (mmap, map_offset) = if let DataSource::Mmap(tm) = data {
            // todo or use typedmap?
            tm.into_inner()
        } else {
            return Err("No file mapped");
        };

        let position = metadata.position.unwrap_or(vector![0.0, 0.0, 0.0]);
        let size = metadata.size.ok_or("No size")?;
        let scale = metadata.scale.ok_or("No scale")?;
        let tf = metadata.tf.ok_or("No tf")?;

        let vol_dims = (size - vector![1, 1, 1]) // side length is n-1 times the point
            .cast::<f32>()
            .component_mul(&scale);
        Ok(StreamVolume {
            position,
            size,
            border: 0,
            scale,
            vol_dims,
            file_map: mmap,
            map_offset,
            tf,
        })
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

        let base = self.get_3d_index(x_low, y_low, z_low) + self.map_offset;

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

    fn sample_at_gradient(&self, pos: Point3<f32>) -> (f32, Vector3<f32>) {
        (0.0, vector![0.0, 0.0, 0.0])
    }

    fn get_data(&self, x: usize, y: usize, z: usize) -> f32 {
        self.get_3d_data(x, y, z)
    }

    fn get_tf(&self) -> TF {
        self.tf
    }
}

#[cfg(test)]
mod test {

    use super::*;

    use crate::ray::Ray;
    use nalgebra::{point, vector};

    #[test]
    fn t1() {
        println!("{:?}", std::env::current_dir());
    }
}
