use nalgebra::{matrix, point, vector, Matrix4, Point3, Vector3};

pub struct TargetCamera {
    pub position: Point3<f32>,
    pub target: Point3<f32>,
    pub resolution: (usize, usize),
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
}

impl TargetCamera {
    pub fn new(width: usize, height: usize) -> TargetCamera {
        TargetCamera {
            position: point![100.0, 100.0, 100.0],
            target: point![34.0, 128.0, 128.0], //target: vector![0.0, 0.0, 0.0],
            resolution: (width, height),
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

pub trait Camera {
    fn get_resolution(&self) -> (usize, usize);

    fn get_position(&self) -> Point3<f32>;

    // return matrix M
    // M * camera_space = world_space
    fn view_matrix(&self) -> Matrix4<f32>;
}
