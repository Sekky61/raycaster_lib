/*
    raycaster_lib
    Author: Michal Majer
    Date: 2022-05-05
*/

use nalgebra::{point, vector, Point3, Vector3, Vector4};

use crate::{common::BoundBox, TF};

use super::{
    vol_builder::{BuildVolume, VolumeMetadata},
    EmptyIndex, Volume,
};

/// Samples are stored in slices, but the slices are converted to floats in preprocessing stage.
/// Viable for small volumes, as it needs to be stored in RAM, preprocessed and takes 4x the memory (4B float vs 1B int sample).
pub struct FloatVolume {
    bound_box: BoundBox, // lower and upper point in world coordinates; lower == position; upper - lower = size
    size: Vector3<usize>,
    data: Vec<f32>,
    tf: TF,
    empty_index: EmptyIndex<4>,
}

impl std::fmt::Debug for FloatVolume {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Volume")
            .field("box", &self.bound_box)
            .field("size", &self.size)
            .field("data len ", &self.data.len())
            .finish()
    }
}

impl FloatVolume {
    fn get_3d_index(&self, x: usize, y: usize, z: usize) -> usize {
        z + y * self.size.z + x * self.size.y * self.size.z
    }

    fn get_3d_data(&self, x: usize, y: usize, z: usize) -> Option<f32> {
        let val = self.data.get(self.get_3d_index(x, y, z));
        val.copied()
    }

    fn get_block_data_half(&self, base: usize) -> Vector4<f32> {
        if base + self.size.z + 1 >= self.data.len() {
            vector![0.0, 0.0, 0.0, 0.0]
        } else {
            vector![
                self.data[base],
                self.data[base + 1],
                self.data[base + self.size.z],
                self.data[base + self.size.z + 1]
            ]
        }
    }
}

impl Volume for FloatVolume {
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

    fn get_size(&self) -> Vector3<usize> {
        self.size
    }

    fn get_tf(&self) -> TF {
        self.tf
    }

    fn get_bound_box(&self) -> BoundBox {
        self.bound_box
    }

    fn get_scale(&self) -> Vector3<f32> {
        vector![1.0, 1.0, 1.0]
    }

    fn set_tf(&mut self, tf: TF) {
        self.tf = tf;
    }

    fn get_name() -> &'static str {
        "FloatVolume"
    }

    fn is_empty(&self, pos: Point3<f32>) -> bool {
        self.empty_index.is_empty(pos)
    }

    fn build_empty_index(&mut self) {
        self.empty_index = EmptyIndex::from_volume(self);
    }
}

impl BuildVolume<u8> for FloatVolume {
    fn build(metadata: VolumeMetadata<u8>) -> Result<FloatVolume, &'static str> {
        println!("Build started");

        let data = metadata.data.ok_or("No volumetric data passed")?;
        let slice = data.get_slice();

        let data: Vec<f32> = slice.iter().map(|&val| val.into()).collect();

        // let data_range_max = data.iter().fold(-10000.0, |cum, &v| f32::max(v, cum));
        // let data_range_min = data.iter().fold(100000.0, |cum, &v| f32::min(v, cum));

        // println!("Build data range: {data_range_min} to {data_range_max}");

        let size = metadata.size.ok_or("No size")?;
        let scale = metadata.scale.unwrap_or_else(|| vector![1.0, 1.0, 1.0]);

        let tf = metadata.tf.ok_or("No transfer function")?;

        let vol_dims = size.map(|v| (v - 1) as f32).component_mul(&scale);

        let position = metadata.position.unwrap_or_else(|| point![0.0, 0.0, 0.0]);

        let bound_box = BoundBox::from_position_dims(position, vol_dims);

        println!("New linear volume, size {size:?} scale {scale:?} bound_box {bound_box:?}");

        let mut volume = FloatVolume {
            bound_box,
            size,
            data,
            tf,
            empty_index: EmptyIndex::dummy(),
        };

        let empty_index = EmptyIndex::from_volume(&volume);
        volume.empty_index = empty_index;

        Ok(volume)
    }
}

#[cfg(test)]
mod test {
    // todo move tests to boundbox

    use nalgebra::{point, vector};

    use super::*;
    use crate::{common::Ray, test_helpers::*};

    #[test]
    fn intersect_works() {
        let vol: FloatVolume = white_volume();
        let bbox = vol.get_bound_box();
        let ray = Ray {
            origin: point![-1.0, -1.0, 0.0],
            direction: vector![1.0, 1.0, 1.0],
        };
        let inter = bbox.intersect(&ray);
        println!("intersection: {:?}", inter);
        assert!(inter.is_some());
    }

    #[test]
    fn intersect_works2() {
        let vol: FloatVolume = white_volume();
        let ray = Ray {
            origin: point![-0.4, 0.73, 0.0],
            direction: vector![1.0, 0.0, 1.0],
        };
        let bbox = vol.get_bound_box();
        let inter = bbox.intersect(&ray);
        println!("intersection: {:?}", inter);
        assert!(inter.is_some());
    }

    #[test]
    fn not_intersecting() {
        let vol: FloatVolume = white_volume();
        let ray = Ray {
            origin: point![200.0, 200.0, 200.0],
            direction: vector![1.0, 0.0, 0.0],
        };
        let bbox = vol.get_bound_box();
        let inter = bbox.intersect(&ray);

        assert!(inter.is_none());
    }
}
