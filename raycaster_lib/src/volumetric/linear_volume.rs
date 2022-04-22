use nalgebra::{point, vector, Point3, Vector3, Vector4};

use crate::{common::BoundBox, TF};

use super::{
    vol_builder::{BuildVolume, VolumeMetadata},
    EmptyIndex, Volume,
};

pub struct LinearVolume {
    bound_box: BoundBox, // lower and upper point in world coordinates; lower == position; upper - lower = size
    size: Vector3<usize>,
    data: Vec<f32>,
    tf: TF,
    empty_index: EmptyIndex<4>,
}

impl std::fmt::Debug for LinearVolume {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Volume")
            .field("box", &self.bound_box)
            .field("size", &self.size)
            .field("data len ", &self.data.len())
            .finish()
    }
}

impl LinearVolume {
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

impl Volume for LinearVolume {
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

    fn get_name(&self) -> &str {
        "LinearVolume"
    }

    fn is_empty(&self, pos: Point3<f32>) -> bool {
        self.empty_index.is_empty(pos)
    }
}

impl BuildVolume<u8> for LinearVolume {
    fn build(metadata: VolumeMetadata<u8>) -> Result<LinearVolume, &'static str> {
        println!("Build started");

        let data = metadata.data.ok_or("No volumetric data passed")?;
        let slice = data.get_slice().ok_or("No data inside datasource")?;
        let offset = metadata.data_offset.unwrap_or(0);

        let data: Vec<f32> = slice[offset..].iter().map(|&val| val.into()).collect();

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

        let mut volume = LinearVolume {
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

// pub struct LinearVolumeHitIterator<'a> {
//     pos: Point3<f32>,
//     step: Vector3<f32>,
//     current_step: u32,
//     n_of_steps: u32,
//     volume: &'a LinearVolume,
// }

// impl<'a> LinearVolumeHitIterator<'a> {
//     pub fn new(
//         pos: Point3<f32>,
//         step: Vector3<f32>,
//         n_of_steps: u32,
//         volume: &'a LinearVolume,
//     ) -> Self {
//         Self {
//             pos,
//             step,
//             current_step: 0,
//             n_of_steps,
//             volume,
//         }
//     }
// }

// impl Iterator for LinearVolumeHitIterator<'_> {
//     fn next(&mut self) -> Option<Self::Item> {
//         while self.current_step < self.n_of_steps {
//             let pos_int: Point3<usize> = self.pos.map(|v| v as usize);

//             // Empty index, loop until end of ray or visible part
//             if self.volume.empty_index.is_empty_int(pos_int) {
//                 // next step
//                 self.pos += self.step;
//                 self.current_step += 1;
//             }

//             // Sample
//             // Gradient

//             let (sample, grad_samples) = self.volume.sample_at_gradient(self.pos);

//             let color = (self.volume.tf)(sample);

//             if color.w == 0.0 {
//                 self.pos += self.step;
//                 self.current_step += 1;
//                 continue;
//             }

//             // Inverted, as low values indicate outside
//             let gradient = vector![
//                 sample - grad_samples.x,
//                 sample - grad_samples.y,
//                 sample - grad_samples.z
//             ];

//             self.pos += self.step;
//             self.current_step += 1;

//             // Construct VolumeHit
//             return Some(VolumeHit::new(color, gradient));
//         }
//         None
//     }
// }

#[cfg(test)]
mod test {
    // todo move tests to boundbox

    use nalgebra::{point, vector};

    use super::*;
    use crate::{common::Ray, test_helpers::*};

    #[test]
    fn intersect_works() {
        let vol: LinearVolume = white_volume();
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
        let vol: LinearVolume = white_volume();
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
        let vol: LinearVolume = white_volume();
        let ray = Ray {
            origin: point![200.0, 200.0, 200.0],
            direction: vector![1.0, 0.0, 0.0],
        };
        let bbox = vol.get_bound_box();
        let inter = bbox.intersect(&ray);

        assert!(inter.is_none());
    }
}
