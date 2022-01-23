use nalgebra::{matrix, point, vector, Matrix4, Point3, Vector2, Vector3};
use sdl2::event::Event;

use super::Camera;

pub struct TargetCamera {
    position: Point3<f32>,
    target: Point3<f32>,
    resolution: (usize, usize),
    drag: Vector2<i32>,
    mouse_down: bool,
}

impl TargetCamera {
    pub fn new(width: usize, height: usize) -> TargetCamera {
        TargetCamera {
            position: point![300.0, 300.0, 300.0],
            target: point![34.0, 128.0, 128.0],
            resolution: (width, height),
            drag: vector![0, 0],
            mouse_down: false,
        }
    }

    pub fn change_pos(&mut self, delta: Vector3<f32>) {
        self.position += delta;
    }

    pub fn set_pos(&mut self, pos: Point3<f32>) {
        self.position = pos;
    }

    pub fn set_target(&mut self, target: Point3<f32>) {
        self.target = target;
    }

    pub fn get_resolution(&self) -> (usize, usize) {
        self.resolution
    }
}

impl Camera for TargetCamera {
    fn get_resolution(&self) -> (usize, usize) {
        self.resolution
    }

    fn view_matrix(&self) -> Matrix4<f32> {
        // calculate camera coord system
        let camera_forward = (self.position - self.target).normalize();
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

    fn get_user_input(&mut self, event: sdl2::event::Event) {
        match event {
            Event::MouseButtonDown { x, y, .. } => {
                self.drag = vector![x, y];
                self.mouse_down = true;
            }
            Event::MouseMotion { x, y, .. } => {
                if !self.mouse_down {
                    return;
                }

                let speed = 0.05;

                let drag_diff = ((x - self.drag.x) as f32, (y - self.drag.y) as f32);
                self.drag = vector![x, y];

                let dif = self.position - self.target;
                let r = dif.magnitude();
                let r = r as f32;
                let mut theta = (dif.z / r).acos();
                let mut phi = dif.y.atan2(dif.x);

                println!(
                    "Current: > drag {:?} dif {} r {} t {} p {}",
                    drag_diff, dif, r, theta, phi
                );

                theta += speed * drag_diff.1;
                phi += speed * drag_diff.0;

                // convert back
                let sphere_offset = vector![
                    r * theta.sin() * phi.cos(),
                    r * theta.sin() * phi.sin(),
                    r * theta.cos()
                ];

                self.set_pos(self.target + sphere_offset);

                println!("New pos > {:?}", self.position);
            }
            Event::MouseButtonUp { x, y, .. } => {
                let drag_diff = (x - self.drag.x, y - self.drag.y);
                self.drag = vector![x, y];
                self.mouse_down = false;
            }
            _ => {}
        }
    }
}
