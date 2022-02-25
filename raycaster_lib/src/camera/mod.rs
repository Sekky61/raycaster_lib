mod perspective_camera;

pub use perspective_camera::PerspectiveCamera;

use crate::ray::Ray;

pub trait Camera {
    // pixel_coord: normalized pixel coords. Range <0;1>^2
    fn get_ray(&self, pixel_coord: (f32, f32)) -> Ray;
}
