use nalgebra::{point, vector, Matrix4, Point3, Vector3};

use crate::common::{BoundBox, Ray, ValueRange};

pub struct Block<const S: usize> {
    pub value_range: ValueRange,
    pub bound_box: BoundBox,
    pub transform: Matrix4<f32>,
    pub data: Vec<f32>,
}

impl<const S: usize> Block<S> {
    pub fn from_data(data: Vec<f32>, bound_box: BoundBox, scale: Vector3<f32>) -> Block<S> {
        assert_eq!(data.len(), S * S * S);
        let value_range = ValueRange::from_iter(&data[..]);

        let scale_inv = vector![1.0, 1.0, 1.0].component_div(&scale);
        let lower_vec = point![0.0, 0.0, 0.0] - bound_box.lower; // todo type workaround

        let transform = Matrix4::identity()
            .append_translation(&lower_vec)
            .append_nonuniform_scaling(&scale_inv);

        Block {
            data,
            bound_box,
            value_range,
            transform,
        }
    }

    // TODO assumes scale == 1
    pub fn transform_ray(&self, ray: &Ray) -> Option<(Ray, f32)> {
        let (t0, t1) = match self.bound_box.intersect(ray) {
            Some(t) => t,
            None => return None,
        };

        let obj_origin = ray.point_from_t(t0);
        let obj_origin = self.transform.transform_point(&obj_origin);

        let t = t1 - t0;

        Some((Ray::from_3(obj_origin, ray.direction), t))
    }

    pub fn get_block_data_half(&self, start_index: usize) -> [f32; 4] {
        [
            self.data[start_index],
            self.data[start_index + 1],
            self.data[start_index + S],
            self.data[start_index + S + 1],
        ]
    }

    pub fn sample_at(&self, pos: Point3<f32>) -> f32 {
        //let data = self.get_block_data(pos);

        let x = pos.x as usize;
        let y = pos.y as usize;
        let z = pos.z as usize;

        let x_t = pos.x.fract();
        let y_t = pos.y.fract();
        let z_t = pos.z.fract();

        let block_offset = self.get_3d_index(x, y, z);

        let first_index = block_offset;
        let second_index = block_offset + S * S;

        let first_data = self.get_block_data_half(first_index);
        let [c000, c001, c010, c011] = first_data;

        let inv_z_t = 1.0 - z_t;
        let inv_y_t = 1.0 - y_t;

        // first plane

        let c00 = c000 * inv_z_t + c001 * z_t; // z low
        let c01 = c010 * inv_z_t + c011 * z_t; // z high
        let c0 = c00 * inv_y_t + c01 * y_t; // point on yz plane

        // second plane

        let second_data = self.get_block_data_half(second_index);
        let [c100, c101, c110, c111] = second_data;

        let c10 = c100 * inv_z_t + c101 * z_t; // z low
        let c11 = c110 * inv_z_t + c111 * z_t; // z high
        let c1 = c10 * inv_y_t + c11 * y_t; // point on yz plane

        c0 * (1.0 - x_t) + c1 * x_t
    }

    fn get_3d_index(&self, x: usize, y: usize, z: usize) -> usize {
        z + y * S + x * S * S
    }
}
