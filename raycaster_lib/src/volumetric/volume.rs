use super::TF;
use crate::ray::Ray;
use nalgebra::{Point3, Vector3};

// Volume assumes f32 data
// Volume is axis aligned
pub trait Volume {
    // get data dimensions
    fn get_size(&self) -> Vector3<usize>;

    // get volume position
    // axis aligned, lowest corner
    fn get_pos(&self) -> Vector3<f32>;

    // get scaled size
    fn get_dims(&self) -> Vector3<f32>;

    // get transfer function
    fn get_tf(&self) -> TF;

    // trilinear interpolation sample, zero if outside
    // pos in volume coordinates
    fn sample_at(&self, pos: Point3<f32>) -> f32;

    // position is inside volume
    fn is_in(&self, pos: &Point3<f32>) -> bool {
        let dims = self.get_dims();
        dims.x > pos.x
            && dims.y > pos.y
            && dims.z > pos.z
            && pos.x > 0.0
            && pos.y > 0.0
            && pos.z > 0.0
    }

    fn get_data(&self, x: usize, y: usize, z: usize) -> f32;

    fn intersect(&self, ray: &Ray) -> Option<(f32, f32)> {
        let dims = self.get_dims();
        // t value of intersection with 6 planes of bounding box
        let t0x = (0.0 - ray.origin.x) / ray.direction.x;
        let t1x = (dims.x - ray.origin.x) / ray.direction.x;
        let t0y = (0.0 - ray.origin.y) / ray.direction.y;
        let t1y = (dims.y - ray.origin.y) / ray.direction.y;
        let t0z = (0.0 - ray.origin.z) / ray.direction.z;
        let t1z = (dims.z - ray.origin.z) / ray.direction.z;

        let tmin = f32::max(
            f32::max(f32::min(t0x, t1x), f32::min(t0y, t1y)),
            f32::min(t0z, t1z),
        );
        let tmax = f32::min(
            f32::min(f32::max(t0x, t1x), f32::max(t0y, t1y)),
            f32::max(t0z, t1z),
        );

        // if tmax < 0, ray (line) is intersecting AABB, but the whole AABB is behind us
        if tmax.is_sign_negative() {
            return None;
        }

        // if tmin > tmax, ray doesn't intersect AABB
        if tmin > tmax {
            return None;
        }

        Some((tmin, tmax))
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

            Some(sample)
        } else {
            None
        }
    }
}
