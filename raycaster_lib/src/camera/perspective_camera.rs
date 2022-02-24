use nalgebra::{matrix, point, vector, Point3, Vector2, Vector3};

use crate::ray::Ray;

use super::Camera;

// up vector = 0,1,0
pub struct PerspectiveCamera {
    position: Point3<f32>,
    direction: Vector3<f32>,
    aspect: f32,
    fov_y: f32,                   // Vertical field of view, in degrees
    img_plane_size: Vector2<f32>, // Calculated from fov_y
    // ray
    dir_00: Vector3<f32>, // Vector from camera point to pixel [0,0]
    du: Vector3<f32>, // Vector between two horizontally neighbouring pixels (example: [0,0] -> [1,0])
    dv: Vector3<f32>, // Vector between two vertically neighbouring pixels (example: [0,0] -> [0,1])
}

impl PerspectiveCamera {
    pub fn new(position: Point3<f32>, direction: Vector3<f32>) -> PerspectiveCamera {
        let up = vector![0.0, 1.0, 0.0];
        let direction = direction.normalize();

        let fov_y = 60.0;
        let mut img_plane_size = vector![0.0, 2.0 * f32::tan(f32::to_radians(0.5 * fov_y))];
        img_plane_size.x = img_plane_size.y; // * aspect, but aspect is 1.0 right now

        let du = direction.cross(&up).normalize() * img_plane_size.x;
        let dv = du.cross(&direction) * img_plane_size.y;
        let dir_00 = direction - 0.5 * du - 0.5 * dv;
        PerspectiveCamera {
            position,
            direction,
            aspect: 1.0,
            fov_y,
            img_plane_size,
            dir_00,
            du,
            dv,
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

    pub fn rotate(&mut self) {}
}

impl Camera for PerspectiveCamera {
    fn get_ray(&self, pixel_coord: (f32, f32)) -> Ray {
        let dir = self.dir_00 + self.du * pixel_coord.0 + self.dv * pixel_coord.1;
        let dir = dir.normalize();
        Ray::from_3(self.position, dir)
    }
}
