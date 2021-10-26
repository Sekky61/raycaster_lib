use crate::ray::Ray;
use nalgebra::Vector3;

// Volume assumes f32 data
pub trait Volume {
    // get data dimensions
    fn get_size(&self) -> Vector3<usize>;

    // get scaled size
    fn get_dims(&self) -> Vector3<f32>;

    // trilinear interpolation sample, zero if outside
    fn sample_at(&self, pos: &Vector3<f32>) -> f32;

    // position is inside volume
    fn is_in(&self, pos: Vector3<f32>) -> bool;

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
}
