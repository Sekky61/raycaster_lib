mod perspective_camera;

pub use perspective_camera::PerspectiveCamera;

use crate::ray::{BoundBox, Ray, ViewportBox};

pub trait Camera {
    // pixel_coord: normalized pixel coords. Range <0;1>^2
    fn get_ray(&self, pixel_coord: (f32, f32)) -> Ray;

    fn project_box(&self, bound_box: BoundBox) -> ViewportBox;

    // todo fn box_distance ?
    fn box_distance(&self, bound_box: &BoundBox) -> f32;
}
