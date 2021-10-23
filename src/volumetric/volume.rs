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

    fn intersect(&self, ray: &Ray) -> Option<(f32, f32)>;

    fn get_data(&self, x: usize, y: usize, z: usize) -> f32;
}
