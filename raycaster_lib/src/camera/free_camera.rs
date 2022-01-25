use nalgebra::{matrix, point, vector, Matrix4, Point3, Vector3};

use super::Camera;

pub struct FreeCamera {
    pub position: Point3<f32>,
    pub direction: Vector3<f32>,
    pub resolution: (usize, usize),
}

impl FreeCamera {
    pub fn new(width: usize, height: usize) -> FreeCamera {
        FreeCamera {
            position: point![100.0, 100.0, 100.0],
            direction: vector![34.0, 128.0, 128.0].normalize(),
            resolution: (width, height),
        }
    }

    pub fn change_pos(&mut self, delta: Vector3<f32>) {
        self.position += delta;
    }

    pub fn set_pos(&mut self, pos: Point3<f32>) {
        self.position = pos;
    }

    pub fn set_direction(&mut self, direction: Vector3<f32>) {
        self.direction = direction.normalize();
    }

    pub fn get_resolution(&self) -> (usize, usize) {
        self.resolution
    }
}

impl Camera for FreeCamera {
    fn get_resolution(&self) -> (usize, usize) {
        self.resolution
    }

    fn view_matrix(&self) -> Matrix4<f32> {
        // calculate camera coord system
        let camera_forward = self.direction;
        let up_vec = vector![0.0, 1.0, 0.0];
        let right = Vector3::cross(&up_vec, &camera_forward);
        let up = Vector3::cross(&camera_forward, &right);

        // cam to world matrix
        matrix![right.x, up.x, camera_forward.x, self.position.x;
                right.y, up.y, camera_forward.y, self.position.y;
                right.z, up.z, camera_forward.z, self.position.z;
                0.0, 0.0, 0.0, 1.0]
    }

    fn get_position(&self) -> Point3<f32> {
        self.position
    }

    fn get_user_input(&mut self, event: &sdl2::event::Event) {}
}
