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
            .append_translation(&-lower_vec)
            .append_nonuniform_scaling(&scale_inv);

        let int = bound_box.intersect(self);
        let obj_origin = match int {
            Some((t0, _t1)) => self.point_from_t(t0),
            None => self.origin,
        };

        let origin = transform.transform_point(&obj_origin);

        let direction = self.direction.component_mul(&scale_inv);

        Ray { origin, direction }
    }
}

#[cfg(test)]
mod test {

    use nalgebra::vector;

    use super::*;

    #[test]
    fn to_object_space() {
        let ray = Ray {
            origin: point![0.0, 0.0, 0.0],
            direction: vector![1.0, 1.0, 1.0],
        };

        let bbox = BoundBox::new(point![1.0, 1.0, 1.0], point![5.0, 5.0, 5.0]);

        let scale = vector![2.0, 1.0, 1.0];

        let obj_ray = ray.transform_to_volume_space(bbox, scale);

        assert_eq!(obj_ray.origin, point![0.0, 0.0, 0.0]);
        assert_eq!(
            obj_ray.direction.normalize(),
            vector![0.5, 1.0, 1.0].normalize()
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

        assert_eq!(obj_ray.origin, point![0.0, 0.25, 0.25]);
        assert_eq!(
            obj_ray.direction.normalize(),
            vector![1.0, -0.25, -0.25].normalize()
        );
    }
}
