use nalgebra::{point, vector, Matrix4, Point3, Vector3};

use super::BoundBox;

/// Ray cast by camera.
/// Main usecase is getting intersections with volumes ([`BoundBox::intersect`]),
/// then iterating over the intersected line segment in steps.
pub struct Ray {
    pub origin: Point3<f32>,
    pub direction: Vector3<f32>,
}

impl Ray {
    /// Construct new ray using `origin` and `direction`.
    /// `direction` must be unit vector.
    pub fn new(origin: Point3<f32>, direction: Vector3<f32>) -> Ray {
        Ray { origin, direction }
    }

    /// Returns point `t` units far from ray origin in ray direction
    pub fn point_from_t(&self, t: f32) -> Point3<f32> {
        self.origin + t * self.direction
    }

    /// Transform ray from world coordinates into volume coordinates.
    ///
    /// # Params
    /// * `bound_box` - Bounding box of volume
    /// * `scale` - Shape of cells in volume
    pub fn transform_to_volume_space(&self, bound_box: BoundBox, scale: Vector3<f32>) -> Ray {
        // Get intersection
        let int = bound_box.intersect(self);
        let obj_origin = match int {
            Some((t0, _t1)) => self.point_from_t(t0),
            None => self.origin,
        };

        // Construct transformation matrix
        let scale_inv = vector![1.0, 1.0, 1.0].component_div(&scale);
        let lower_vec = bound_box.lower - point![0.0, 0.0, 0.0];

        let transform = Matrix4::identity()
            .append_translation(&-lower_vec)
            .append_nonuniform_scaling(&scale_inv);

        // Construct transformed vector
        let origin = transform.transform_point(&obj_origin);
        let direction = self.direction.component_mul(&scale_inv);
        Ray { origin, direction }
    }
}

#[cfg(test)]
mod test {

    use nalgebra::vector;

    use super::*;

    fn compare_float_vec(actual: Vector3<f32>, expected: Vector3<f32>, error: f32) {
        let err = f32::abs(actual.x - expected.x);
        assert!(err < error);
        let err = f32::abs(actual.y - expected.y);
        assert!(err < error);
        let err = f32::abs(actual.z - expected.z);
        assert!(err < error);
    }

    fn compare_float_point(actual: Point3<f32>, expected: Point3<f32>, error: f32) {
        let err = f32::abs(actual.x - expected.x);
        assert!(err < error);
        let err = f32::abs(actual.y - expected.y);
        assert!(err < error);
        let err = f32::abs(actual.z - expected.z);
        assert!(err < error);
    }

    #[test]
    fn to_object_space() {
        let ray = Ray {
            origin: point![0.0, 0.0, 0.0],
            direction: vector![1.0, 1.0, 1.0],
        };

        let bbox = BoundBox::new(point![1.0, 1.0, 1.0], point![5.0, 5.0, 5.0]);

        let scale = vector![2.0, 1.0, 1.0];

        let obj_ray = ray.transform_to_volume_space(bbox, scale);

        compare_float_point(obj_ray.origin, point![0.0, 0.0, 0.0], 0.01);
        compare_float_vec(
            obj_ray.direction.normalize(),
            vector![0.5, 1.0, 1.0].normalize(),
            0.01,
        );
    }

    #[test]
    fn to_object_space_2() {
        let ray = Ray {
            origin: point![5.0, 0.5, 0.5],
            direction: vector![2.0, -0.5, -0.5],
        };

        let bbox = BoundBox::new(point![6.0, 0.0, 0.0], point![7.0, 1.0, 1.0]);

        let scale = vector![1.0, 1.0, 1.0];

        let obj_ray = ray.transform_to_volume_space(bbox, scale);

        compare_float_point(obj_ray.origin, point![0.0, 0.25, 0.25], 0.01);
        compare_float_vec(
            obj_ray.direction.normalize(),
            vector![1.0, -0.25, -0.25].normalize(),
            0.01,
        );
    }
}
