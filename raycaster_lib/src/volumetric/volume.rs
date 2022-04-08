use crate::common::{BoundBox, Ray};

use crate::TF;
use nalgebra::{point, vector, Matrix4, Point3, Vector3};

// Volume assumes f32 data
// Volume is axis aligned
pub trait Volume: Send {
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

    // set transfer function
    fn set_tf(&mut self, tf: TF);

    // trilinear interpolation sample, zero if outside
    // pos in volume coordinates
    fn sample_at(&self, pos: Point3<f32>) -> f32;

    // trilinear interpolation sample, zero if outside
    // pos in volume coordinates
    fn sample_at_gradient(&self, pos: Point3<f32>) -> (f32, Vector3<f32>) {
        // Default implementation, can be replaced with a more effective one for concrete volume types
        let sample = self.sample_at(pos);
        let grad_dir = 0.4; // todo take into account voxel shape

        let size = self.get_size().map(|v| v as f32);
        // todo change from sampling to gradient compute and optimise

        let sample_x = if pos.x + grad_dir > size.x {
            0.0 // background value
        } else {
            self.sample_at(pos + vector![grad_dir, 0.0, 0.0])
        };

        let sample_y = if pos.y + grad_dir > size.y {
            0.0 // background value
        } else {
            self.sample_at(pos + vector![0.0, grad_dir, 0.0])
        };

        let sample_z = if pos.z + grad_dir > size.z {
            0.0 // background value
        } else {
            self.sample_at(pos + vector![0.0, 0.0, grad_dir])
        };

        let grad_samples = vector![sample_x, sample_y, sample_z];

        (sample, grad_samples)
    }

    fn get_bound_box(&self) -> BoundBox; // todo ref

    fn get_scale(&self) -> Vector3<f32>; // todo ref

    fn intersect_transform(&self, ray: &Ray) -> Option<(Ray, f32)> {
        let bbox = self.get_bound_box();

        let (t0, t1) = bbox.intersect(ray)?;

        let scale_inv = vector![1.0, 1.0, 1.0].component_div(&self.get_scale());
        let lower_vec = bbox.lower - point![0.0, 0.0, 0.0];

        let transform = Matrix4::identity()
            .append_translation(&-lower_vec)
            .append_nonuniform_scaling(&scale_inv);

        let obj_origin = ray.point_from_t(t0);

        let origin = transform.transform_point(&obj_origin);

        let direction = ray.direction.component_mul(&scale_inv);
        let direction = direction.normalize();

        let obj_ray = Ray::from_3(origin, direction);

        Some((obj_ray, t1 - t0))
    }

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
