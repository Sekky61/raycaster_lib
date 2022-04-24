use nalgebra::{point, vector, Matrix4, Point3, Vector3, Vector4};

use crate::{
    common::{BoundBox, Ray, ValueRange},
    TF,
};

use super::{EmptyIndex, Volume};

pub struct FloatBlock {
    pub block_side: usize,
    pub value_range: ValueRange,
    pub bound_box: BoundBox,
    pub transform: Matrix4<f32>,
    pub data: Vec<f32>,
    empty_index: EmptyIndex<4>,
}

impl FloatBlock {
    pub fn from_data(
        data: Vec<f32>,
        bound_box: BoundBox,
        scale: Vector3<f32>,
        block_side: usize,
        tf: TF,
    ) -> FloatBlock {
        // todo boundbox and scale has redundant info
        assert_eq!(data.len(), block_side.pow(3));
        let value_range = ValueRange::from_samples(&data[..]);

        let scale_inv = vector![1.0, 1.0, 1.0].component_div(&scale);
        let lower_vec = point![0.0, 0.0, 0.0] - bound_box.lower; // todo type workaround

        let transform = Matrix4::identity()
            .append_translation(&lower_vec)
            .append_nonuniform_scaling(&scale_inv);

        let mut block = FloatBlock {
            data,
            bound_box,
            value_range,
            transform,
            block_side,
            empty_index: EmptyIndex::dummy(),
        };

        block.empty_index = EmptyIndex::<4>::from_volume_without_tf(&block, tf);
        block
    }

    pub fn get_block_data_half(&self, start_index: usize) -> Vector4<f32> {
        if start_index + self.block_side + 1 >= self.data.len() {
            vector![0.0, 0.0, 0.0, 0.0]
        } else {
            vector![
                self.data[start_index],
                self.data[start_index + 1],
                self.data[start_index + self.block_side],
                self.data[start_index + self.block_side + 1]
            ]
        }
    }

    fn get_3d_index(&self, x: usize, y: usize, z: usize) -> usize {
        z + y * self.block_side + x * self.block_side * self.block_side
    }
}

// todo subvolume trait?
impl Volume for FloatBlock {
    // A more optimal specialization
    fn transform_ray(&self, ray: &Ray) -> Option<(Ray, f32)> {
        // TODO assumes scale == 1
        let (t0, t1) = match self.bound_box.intersect(ray) {
            Some(t) => t,
            None => return None,
        };

        let obj_origin = ray.point_from_t(t0);
        let obj_origin = self.transform.transform_point(&obj_origin);

        let t = t1 - t0;

        Some((Ray::new(obj_origin, ray.direction), t))
    }

    fn get_size(&self) -> Vector3<usize> {
        vector![self.block_side, self.block_side, self.block_side]
    }

    fn get_tf(&self) -> TF {
        unimplemented!()
    }

    fn set_tf(&mut self, _tf: TF) {
        unimplemented!()
    }

    fn sample_at(&self, pos: Point3<f32>) -> f32 {
        let x = pos.x as usize;
        let y = pos.y as usize;
        let z = pos.z as usize;

        let x_t = pos.x.fract();
        let y_t = pos.y.fract();
        let z_t = pos.z.fract();

        let block_offset = self.get_3d_index(x, y, z);

        let first_index = block_offset;
        let second_index = block_offset + self.block_side * self.block_side;

        // first plane
        // c000, c001, c010, c011
        let mut x_low_vec = self.get_block_data_half(first_index);

        // second plane
        // c100, c101, c110, c111
        let mut x_hi_vec = self.get_block_data_half(second_index);

        x_low_vec *= 1.0 - x_t;
        x_hi_vec *= x_t;

        //x plane
        x_low_vec += x_hi_vec;
        let inv_y_t = 1.0 - y_t;
        x_low_vec.component_mul_assign(&vector![inv_y_t, inv_y_t, y_t, y_t]);

        // y line
        let c0: f32 = x_low_vec.x + x_low_vec.z;
        let c1: f32 = x_low_vec.y + x_low_vec.w;

        c0 * (1.0 - z_t) + c1 * z_t
    }

    fn get_bound_box(&self) -> BoundBox {
        self.bound_box
    }

    fn get_scale(&self) -> Vector3<f32> {
        unimplemented!()
    }

    fn get_data(&self, x: usize, y: usize, z: usize) -> Option<f32> {
        let index = self.get_3d_index(x, y, z);
        self.data.get(index).cloned()
    }

    fn get_name() -> &'static str {
        "FloatBlock"
    }

    fn is_empty(&self, pos: Point3<f32>) -> bool {
        self.empty_index.is_empty(pos)
    }

    fn build_empty_index(&mut self) {
        self.empty_index = EmptyIndex::from_volume(self);
    }
}
