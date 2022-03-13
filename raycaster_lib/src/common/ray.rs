use nalgebra::{point, vector, Matrix4, Point3, Scale3, Translation3, Vector3};

use super::BoundBox;

// Todo rename to common / types

pub struct Ray {
    // todo t parameter
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

    pub fn transform_to_volume_space(&self, bound_box: BoundBox, scale: Vector3<f32>) -> Ray {
        let scale_inv = vector![1.0, 1.0, 1.0].component_div(&scale);
        let lower_vec = bound_box.lower - point![0.0, 0.0, 0.0];

        let transform = Matrix4::identity()
            .append_translation(&lower_vec)
            .append_nonuniform_scaling(&scale_inv);

        let origin = transform.transform_point(&self.origin);

        let direction = self.direction.component_mul(&scale_inv);

        Ray { origin, direction }
    }
}
