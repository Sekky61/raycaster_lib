use nalgebra::{Matrix4, Point3};

mod free_camera;
mod target_camera;

pub use free_camera::FreeCamera;
pub use target_camera::TargetCamera;

pub trait Camera {
    fn get_resolution(&self) -> (usize, usize);

    fn get_position(&self) -> Point3<f32>;

    // return matrix M
    // M * camera_space = world_space
    fn view_matrix(&self) -> Matrix4<f32>;

    // todo general
    fn get_user_input(&mut self, event: sdl2::event::Event);
}
