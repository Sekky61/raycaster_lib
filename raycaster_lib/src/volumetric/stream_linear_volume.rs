use memmap::Mmap;
use nalgebra::{point, vector, Point3, Vector3, Vector4};

use crate::{
    common::BoundBox,
    volumetric::{vol_builder::DataSource, EmptyIndex},
    TF,
};

use super::{vol_builder::VolumeMetadata, BuildVolume, Volume};

#[derive(Debug)]
pub struct StreamLinearVolume {
    // todo rename to streamlinearvolume
    bound_box: BoundBox, // todo empty index
    size: Vector3<usize>,
    empty_index: EmptyIndex<4>,
    file_map: Mmap,
    map_offset: usize,
    tf: TF,
}

impl StreamLinearVolume {
    fn get_3d_index(&self, x: usize, y: usize, z: usize) -> usize {
        z + y * self.size.z + x * self.size.y * self.size.z
    }

    fn get_3d_data(&self, x: usize, y: usize, z: usize) -> Option<f32> {
        let index = self.map_offset + self.get_3d_index(x, y, z);
        let buf: &[u8] = self.file_map.as_ref();
        buf.get(index).map(|v| *v as f32)
    }

    fn get_block_data_half(&self, base: usize) -> Vector4<f32> {
        let buf: &[u8] = self.file_map.as_ref();
        if base + self.size.z + 1 >= buf.len() {
            vector![0.0, 0.0, 0.0, 0.0]
        } else {
            vector![
                buf[base] as f32,
                buf[base + 1] as f32,
                buf[base + self.size.z] as f32,
                buf[base + self.size.z + 1] as f32
            ]
        }
    }
}

impl BuildVolume<u8> for StreamLinearVolume {
    fn build(metadata: VolumeMetadata<u8>) -> Result<StreamLinearVolume, &'static str> {
        println!("Build started");

        let data = metadata.data.ok_or("No data")?;

        let (mmap, map_offset) = if let DataSource::Mmap(tm) = data {
            // todo or use typedmap?
            tm.into_inner()
        } else {
            return Err("No file mapped");
        };

        let position = metadata.position.unwrap_or_else(|| point![0.0, 0.0, 0.0]);
        let size = metadata.size.ok_or("No size")?;
        let scale = metadata.scale.ok_or("No scale")?;
        let tf = metadata.tf.ok_or("No tf")?;

        let vol_dims = (size - vector![1, 1, 1]) // side length is n-1 times the point
            .cast::<f32>()
            .component_mul(&scale);

        let bound_box = BoundBox::from_position_dims(position, vol_dims);

        println!(
            "Constructed StreamLinearVolume ({}x{}x{})",
            size.x, size.y, size.z
        );

        let mut volume = StreamLinearVolume {
            bound_box,
            size,
            file_map: mmap,
            map_offset,
            tf,
            empty_index: EmptyIndex::dummy(),
        };

        let empty_index = EmptyIndex::from_volume(&volume);
        volume.empty_index = empty_index;

        Ok(volume)
    }
}

impl Volume for StreamLinearVolume {
    fn get_size(&self) -> Vector3<usize> {
        self.size
    }

    fn sample_at(&self, pos: Point3<f32>) -> f32 {
        let x = pos.x as usize;
        let y = pos.y as usize;
        let z = pos.z as usize;

        let x_t = pos.x.fract();
        let y_t = pos.y.fract();
        let z_t = pos.z.fract();

        let block_offset = self.get_3d_index(x, y, z);

        let first_index = block_offset;
        let second_index = first_index + self.size.z * self.size.y;

        // first plane
        // c000, c001, c010, c011
        let mut x_low_vec = self.get_block_data_half(first_index);

        // second plane
        // c100, c101, c110, c111
        let mut x_hi_vec = self.get_block_data_half(second_index);

        x_low_vec *= 1.0 - x_t;
        x_hi_vec *= x_t;

        //x plane
        x_low_vec += x_hi_vec;
        let inv_y_t = 1.0 - y_t;
        x_low_vec.component_mul_assign(&vector![inv_y_t, inv_y_t, y_t, y_t]);

        // y line
        let c0: f32 = x_low_vec.x + x_low_vec.z;
        let c1: f32 = x_low_vec.y + x_low_vec.w;

        c0 * (1.0 - z_t) + c1 * z_t
    }

    fn get_data(&self, x: usize, y: usize, z: usize) -> Option<f32> {
        self.get_3d_data(x, y, z)
    }

    fn get_tf(&self) -> TF {
        self.tf
    }

    fn set_tf(&mut self, tf: TF) {
        self.tf = tf;
    }

    fn get_bound_box(&self) -> BoundBox {
        self.bound_box
    }

    fn get_scale(&self) -> Vector3<f32> {
        vector![1.0, 1.0, 1.0]
    }

    fn get_name(&self) -> &str {
        "StreamLinearVolume"
    }

    fn is_empty(&self, pos: Point3<f32>) -> bool {
        self.empty_index.is_empty(pos)
    }
}
