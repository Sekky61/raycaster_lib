use crate::ray::Ray;
use nalgebra::Vector3;

pub trait Volume {
    // get scaled size
    fn get_dims(&self) -> Vector3<f32>;

    // trilinear interpolation sample, zero if outside
    fn sample_at(&self, pos: Vector3<f32>) -> f32;

    // position is inside volume
    fn is_in(&self, pos: Vector3<f32>) -> bool;

    fn intersect(&self, ray: &Ray) -> Option<(f32, f32)>;
}

// R G B A -- A <0;1>
pub fn transfer_function(sample: f32) -> (f32, f32, f32, f32) {
    if sample > 180.0 {
        (60.0, 230.0, 40.0, 0.3)
    } else if sample > 70.0 {
        (230.0, 10.0, 10.0, 0.3)
    } else if sample > 50.0 {
        (10.0, 20.0, 100.0, 0.1)
    } else if sample > 5.0 {
        (10.0, 10.0, 40.0, 0.05)
    } else {
        (0.0, 0.0, 0.0, 0.0)
    }
}
