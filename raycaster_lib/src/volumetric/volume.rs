use crate::common::{BoundBox, Ray};

use super::TF;
use nalgebra::{vector, Point3, Vector3};

// Volume assumes f32 data
// Volume is axis aligned
pub trait Volume {
    // get data dimensions
    fn get_size(&self) -> Vector3<usize>;

    // get volume position
    // axis aligned, lowest corner
    fn get_pos(&self) -> Point3<f32> {
        self.get_bound_box().lower
    }

    // get scaled size
    fn get_dims(&self) -> Vector3<f32> {
        self.get_bound_box().dims()
    }

    // get transfer function
    fn get_tf(&self) -> TF;

    // trilinear interpolation sample, zero if outside
    // pos in volume coordinates
    fn sample_at(&self, pos: Point3<f32>) -> f32;

    // trilinear interpolation sample, zero if outside
    // pos in volume coordinates
    fn sample_at_gradient(&self, pos: Point3<f32>) -> (f32, Vector3<f32>) {
        // Default implementation, can be replaced with a more effective one for concrete volume types
        let sample = self.sample_at(pos);
        let mut grad_dir = vector![0.05, 0.05, 0.05]; // todo get scale / voxel shape

        let size = self.get_size().map(|v| v as f32);

        if pos.x + grad_dir.x > size.x {
            grad_dir.x *= -1.0;
        }

        if pos.y + grad_dir.y > size.y {
            grad_dir.y *= -1.0;
        }

        if pos.z + grad_dir.z > size.z {
            grad_dir.z *= -1.0;
        }

        let sample_x = self.sample_at(pos + vector![grad_dir.x, 0.0, 0.0]);
        let sample_y = self.sample_at(pos + vector![0.0, grad_dir.y, 0.0]);
        let sample_z = self.sample_at(pos + vector![0.0, 0.0, grad_dir.z]);
        let grad_samples = vector![sample_x, sample_y, sample_z];

        (sample, grad_samples)
    }

    fn get_bound_box(&self) -> BoundBox; // todo ref

    // position is inside volume
    fn is_in(&self, pos: &Point3<f32>) -> bool {
        self.get_bound_box().is_in(pos)
    }

    // For building and tests, mostly
    fn get_data(&self, x: usize, y: usize, z: usize) -> Option<f32>;

    fn intersect(&self, ray: &Ray) -> Option<(f32, f32)> {
        self.get_bound_box().intersect(ray)
    }

    fn get_block(&self, side: usize, base: Point3<usize>) -> VolumeBlockIter<Self> {
        VolumeBlockIter {
            volume: self,
            base,
            side,
            iter_progress: 0,
        }
    }
}

// Warning: Returns None if block element is outside the volume
// In other words, returned None option does not necesarily mean the iterator is exhausted
pub struct VolumeBlockIter<'a, V>
where
    V: 'a + Volume + ?Sized,
{
    volume: &'a V,
    base: Point3<usize>,
    side: usize,
    iter_progress: usize,
}

impl<'a, V: Volume> Iterator for VolumeBlockIter<'a, V> {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let block_len = self.side * self.side * self.side;

        if self.iter_progress < block_len {
            let offset_x = self.iter_progress / self.side / self.side;
            let offset_y = self.iter_progress / self.side;
            let offset_z = self.iter_progress % self.side;

            let sample = self.volume.get_data(
                self.base.x + offset_x,
                self.base.y + offset_y,
                self.base.z + offset_z,
            );

            self.iter_progress += 1;

            sample
        } else {
            None
        }
    }
}
