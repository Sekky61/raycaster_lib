use nalgebra::{Point3, Vector3};

pub struct Ray {
    pub origin: Point3<f32>,
    pub direction: Vector3<f32>, // todo Unit
}

impl Ray {
    pub fn from_3(origin: Point3<f32>, direction: Vector3<f32>) -> Ray {
        Ray { origin, direction }
    }

    pub fn point_from_t(&self, t: f32) -> Point3<f32> {
        self.origin + t * self.direction
    }

    pub fn get_direction(&self) -> Vector3<f32> {
        self.direction
    }
}
