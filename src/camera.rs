use nalgebra::{matrix, vector, Matrix4, Vector3};

pub struct Camera {
    pub position: Vector3<f32>,
    pub target: Vector3<f32>,
    pub resolution: (usize, usize),
}

impl Camera {
    pub fn new(width: usize, height: usize) -> Camera {
        Camera {
            position: vector![-4.0, 108.0, -85.0],
            target: vector![34.0, 128.0, 128.0], //target: vector![0.0, 0.0, 0.0],
            resolution: (width, height),
        }
    }

    pub fn change_pos(&mut self, delta: Vector3<f32>) {
        self.position += delta;
    }

    pub fn set_pos(&mut self, pos: Vector3<f32>) {
        self.position = pos;
    }

    pub fn get_resolution(&self) -> (usize, usize) {
        self.resolution
    }

    pub fn get_look_at_matrix(&self) -> Matrix4<f32> {
        let camera_forward = (self.position - self.target).normalize();
        let up_vec = vector![0.0, 1.0, 0.0];
        let right = Vector3::cross(&up_vec, &camera_forward);
        let up = Vector3::cross(&camera_forward, &right);

        // cam to world
        matrix![right.x, up.x, camera_forward.x, self.position.x;
                                                            right.y, up.y, camera_forward.y, self.position.y;
                                                            right.z,up.z,camera_forward.z, self.position.z;
                                                            0.0,0.0,0.0, 1.0]
    }
}
