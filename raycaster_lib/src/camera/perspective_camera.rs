use nalgebra::{vector, Point3, Rotation3, Vector2, Vector3};

use crate::ray::{BoundBox, Ray, ViewportBox};

use super::Camera;

// up vector = 0,1,0
pub struct PerspectiveCamera {
    position: Point3<f32>,
    up: Vector3<f32>,
    right: Vector3<f32>,
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

        let right = direction.cross(&up); // todo normalize and unit?
        let up = right.cross(&direction);

        let fov_y = 60.0;
        let mut img_plane_size = vector![0.0, 2.0 * f32::tan(f32::to_radians(0.5 * fov_y))];
        img_plane_size.x = img_plane_size.y; // * aspect, but aspect is 1.0 right now

        let du: Vector3<f32> = img_plane_size.x * direction.cross(&up).normalize();
        let dv: Vector3<f32> = img_plane_size.y * du.cross(&direction);
        let dir_00 = direction - 0.5 * du - 0.5 * dv;
        PerspectiveCamera {
            position,
            up,
            right,
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

    pub fn change_pos_plane(&mut self, x: f32, y: f32) {
        self.position += x * self.right + y * self.up;
    }

    pub fn change_pos_view_dir(&mut self, delta: f32) {
        self.position += delta * self.direction;
    }

    pub fn look_around(&mut self, x: f32, y: f32) {
        self.direction += self.du * x + self.dv * y;
        self.recalc_plane();
    }

    pub fn change_pos_matrix(&mut self, matrix: Rotation3<f32>) {
        self.position = matrix * self.position;
        self.direction = matrix * self.direction;

        self.recalc_plane();
    }

    fn recalc_plane(&mut self) {
        let up = vector![0.0, 1.0, 0.0];
        self.right = self.direction.cross(&up); // todo normalize and unit?
        self.up = self.right.cross(&self.direction);

        self.du = self.img_plane_size.x * self.direction.cross(&self.up).normalize();
        self.dv = self.img_plane_size.y * self.du.cross(&self.direction);
        self.dir_00 = self.direction - 0.5 * self.du - 0.5 * self.dv;
    }

    pub fn set_pos(&mut self, pos: Point3<f32>) {
        self.position = pos;
    }

    pub fn set_direction(&mut self, direction: Vector3<f32>) {
        self.direction = direction.normalize();
    }
}

impl Camera for PerspectiveCamera {
    fn get_ray(&self, pixel_coord: (f32, f32)) -> Ray {
        let dir = self.dir_00 + self.du * pixel_coord.0 + self.dv * pixel_coord.1;
        let dir = dir.normalize();
        Ray::from_3(self.position, dir)
    }

    fn project_box(&self, bound_box: BoundBox) -> ViewportBox {
        let mut viewbox = ViewportBox::new();

        for point in bound_box {
            let v = point - self.position;
            let n = v.normalize();
            let neg_n = -n;
            let neg_dir = -self.direction;

            let den = neg_n.dot(&neg_dir);
            if den != 0.0 {
                let t = 1.0 / den;
                let screen_dir = n * t - self.dir_00;
                let x = screen_dir.dot(&self.du);
                let y = screen_dir.dot(&self.dv);
                viewbox.add_point(x, y);
            }
        }

        viewbox
    }
}
