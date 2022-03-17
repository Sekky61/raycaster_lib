use nalgebra::Point3;

use crate::common::{BoundBox, ValueRange};

// TODO can be moved inside a crate feature

pub struct BlockGen<const S: usize>
where
    [f32; S * S * S]: Sized,
{
    pub value_range: ValueRange,
    pub bound_box: BoundBox,
    pub data: [f32; S * S * S],
}

// impl<const S: usize> BlockGen<S>
// where
//     [f32; S * S * S]: Sized,
// {
//     pub fn from_data(data: [f32; S * S * S], bound_box: BoundBox) -> BlockGen<S> {
//         let value_range = ValueRange::from_iter(&data);
//         BlockGen {
//             data,
//             bound_box,
//             value_range,
//         }
//     }

//     fn get_block_data_half(&self, start_index: usize) -> [f32; 4] {
//         [
//             self.data[start_index],
//             self.data[start_index + 1],
//             self.data[start_index + S],
//             self.data[start_index + S + 1],
//         ]
//     }

//     pub fn sample_at(&self, pos: Point3<f32>) -> f32 {
//         //let data = self.get_block_data(pos);

//         let x = pos.x as usize;
//         let y = pos.y as usize;
//         let z = pos.z as usize;

//         let x_t = pos.x.fract();
//         let y_t = pos.y.fract();
//         let z_t = pos.z.fract();

//         let block_offset = self.get_3d_index(x, y, z);

//         let first_index = block_offset;
//         let second_index = block_offset + S * S;

//         let first_data = self.get_block_data_half(first_index);
//         let [c000, c001, c010, c011] = first_data;

//         let inv_z_t = 1.0 - z_t;
//         let inv_y_t = 1.0 - y_t;

//         // first plane

//         let c00 = c000 * inv_z_t + c001 * z_t; // z low
//         let c01 = c010 * inv_z_t + c011 * z_t; // z high
//         let c0 = c00 * inv_y_t + c01 * y_t; // point on yz plane

//         // second plane

//         let second_data = self.get_block_data_half(second_index);
//         let [c100, c101, c110, c111] = second_data;

//         let c10 = c100 * inv_z_t + c101 * z_t; // z low
//         let c11 = c110 * inv_z_t + c111 * z_t; // z high
//         let c1 = c10 * inv_y_t + c11 * y_t; // point on yz plane

//         c0 * (1.0 - x_t) + c1 * x_t
//     }

//     fn get_3d_index(&self, x: usize, y: usize, z: usize) -> usize {
//         z + y * S + x * S * S
//     }
// }

// #[cfg(test)]
// mod test {

//     use nalgebra::{point, vector};

//     use super::*;

//     #[test]
//     fn construction() {
//         // let mut data = [0.0; 3 * 3 * 3];
//         // data[2] = 1.9;
//         // data[9] = 1.8;
//         // data[20] = 0.0;
//         // let bbox = BoundBox::new(point![0.0, 0.0, 0.0], point![1.0, 1.0, 1.0]);
//         // let block = BlockGen::<3>::from_data(data, bbox);

//         //assert_eq!(block.value_range.limits(), (0.0, 1.9));
//     }
// }
