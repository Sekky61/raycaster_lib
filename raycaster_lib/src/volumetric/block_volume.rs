use nalgebra::{point, vector, Point3, Vector3};

use crate::common::{blockify, BoundBox, ValueRange};

use super::{
    vol_builder::{BuildVolume, VolumeMetadata},
    Volume, TF,
};

const BLOCK_SIDE: usize = 16;
const BLOCK_OVERLAP: usize = 1;
const BLOCK_DATA_LEN: usize = BLOCK_SIDE.pow(3);

pub struct BlockVolume {
    bound_box: BoundBox,
    data_size: Vector3<usize>,
    block_size: Vector3<usize>, // Number of blocks in structure (.data)
    pub data: Vec<Block>,
    tf: TF,
}

impl BlockVolume {
    // returns (block index, block offset)
    fn get_indexes(&self, x: usize, y: usize, z: usize) -> (usize, usize) {
        let jump_per_block = BLOCK_SIDE - BLOCK_OVERLAP;
        //assert_ne!(jump_per_block, 0);
        let block_offset = (z % jump_per_block)
            + (y % jump_per_block) * BLOCK_SIDE
            + (x % jump_per_block) * BLOCK_SIDE * BLOCK_SIDE;
        let block_index = (z / jump_per_block)
            + (y / jump_per_block) * self.block_size.z
            + (x / jump_per_block) * self.block_size.y * self.block_size.z;
        (block_index, block_offset)
    }

    // get voxel
    fn get_3d_data(&self, x: usize, y: usize, z: usize) -> Option<f32> {
        let (block_index, block_offset) = self.get_indexes(x, y, z);
        match self.data.get(block_index) {
            Some(b) => b.data.get(block_offset).copied(),
            None => None,
        }
    }
}

pub struct Block {
    pub value_range: ValueRange,
    pub bound_box: BoundBox,
    pub data: [f32; BLOCK_DATA_LEN],
}

impl Block {
    pub fn from_data(data: [f32; BLOCK_DATA_LEN], bound_box: BoundBox) -> Block {
        let value_range = ValueRange::from_iter(&data);
        Block {
            data,
            bound_box,
            value_range,
        }
    }

    fn get_block_data_half(&self, start_index: usize) -> [f32; 4] {
        [
            self.data[start_index],
            self.data[start_index + 1],
            self.data[start_index + BLOCK_SIDE],
            self.data[start_index + BLOCK_SIDE + 1],
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
        let second_index = block_offset + BLOCK_SIDE * BLOCK_SIDE;

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
        z + y * BLOCK_SIDE + x * BLOCK_SIDE * BLOCK_SIDE
    }
}

impl Volume for BlockVolume {
    fn get_size(&self) -> Vector3<usize> {
        self.data_size
    }

    fn sample_at(&self, pos: Point3<f32>) -> f32 {
        //let data = self.get_block_data(pos);

        let x = pos.x as usize;
        let y = pos.y as usize;
        let z = pos.z as usize;

        let x_t = pos.x.fract();
        let y_t = pos.y.fract();
        let z_t = pos.z.fract();

        let (block_index, block_offset) = self.get_indexes(x, y, z);

        let block = &self.data[block_index];
        let first_index = block_offset;
        let second_index = block_offset + BLOCK_SIDE * BLOCK_SIDE;

        let first_data = block.get_block_data_half(first_index);
        let [c000, c001, c010, c011] = first_data;

        let inv_z_t = 1.0 - z_t;
        let inv_y_t = 1.0 - y_t;

        // first plane

        let c00 = c000 * inv_z_t + c001 * z_t; // z low
        let c01 = c010 * inv_z_t + c011 * z_t; // z high
        let c0 = c00 * inv_y_t + c01 * y_t; // point on yz plane

        // second plane

        let second_data = block.get_block_data_half(second_index);
        let [c100, c101, c110, c111] = second_data;

        let c10 = c100 * inv_z_t + c101 * z_t; // z low
        let c11 = c110 * inv_z_t + c111 * z_t; // z high
        let c1 = c10 * inv_y_t + c11 * y_t; // point on yz plane

        c0 * (1.0 - x_t) + c1 * x_t
    }

    fn get_data(&self, x: usize, y: usize, z: usize) -> Option<f32> {
        self.get_3d_data(x, y, z)
    }

    fn get_tf(&self) -> super::TF {
        self.tf
    }

    fn get_bound_box(&self) -> BoundBox {
        self.bound_box
    }

    fn get_scale(&self) -> Vector3<f32> {
        vector![1.0, 1.0, 1.0] // todo
    }
}

impl BuildVolume<u8> for BlockVolume {
    fn build(metadata: VolumeMetadata<u8>) -> Result<BlockVolume, &'static str> {
        let position = metadata.position.unwrap_or_else(|| point![0.0, 0.0, 0.0]);
        let size = metadata.size.ok_or("No size")?;
        let scale = metadata.scale.ok_or("No scale")?;
        let data = metadata.data.ok_or("No data")?;
        let offset = metadata.data_offset.unwrap_or(0);
        let tf = metadata.tf.ok_or("No transfer function")?;

        let vol_dims = (size - vector![1, 1, 1]) // side length is n-1 times the point
            .cast::<f32>();
        let vol_dims = (vol_dims - vector![0.1, 0.1, 0.1]).component_mul(&scale); // todo workaround

        let bound_box = BoundBox::from_position_dims(position, vol_dims);

        let mut blocks = vec![];

        let step_size = BLOCK_SIDE - BLOCK_OVERLAP;
        //let block_size = size.map(|v| ((v - 1) / step_size) as usize);

        let block_size = blockify(size, BLOCK_SIDE, BLOCK_OVERLAP);

        let slice = &data.get_slice().ok_or("No data in datasource")?[offset..];

        for x in 0..block_size.x {
            for y in 0..block_size.y {
                for z in 0..block_size.z {
                    let block_start = step_size * point![x, y, z];
                    let block_data = get_block_data(slice, size, block_start);
                    let block_bound_box = get_bound_box(position, scale, block_start);
                    let block = Block::from_data(block_data, block_bound_box);
                    blocks.push(block);
                }
            }
        }

        println!(
            "Built {} blocks of dims {BLOCK_SIDE} blocks ({},{},{}) -> ({},{},{})",
            blocks.len(),
            size.x,
            size.y,
            size.z,
            block_size.x,
            block_size.y,
            block_size.z,
        );

        println!("{size}");

        Ok(BlockVolume {
            bound_box,
            data_size: size,
            block_size,
            data: blocks,
            tf,
        })
    }
}

fn get_bound_box(
    vol_position: Point3<f32>,
    vol_scale: Vector3<f32>,
    block_start: Point3<usize>,
) -> BoundBox {
    let block_lower = vector![
        block_start.x as f32,
        block_start.y as f32,
        block_start.z as f32
    ];
    let block_pos = vol_position + block_lower.component_mul(&vol_scale);
    let block_dims =
        vector![BLOCK_SIDE as f32, BLOCK_SIDE as f32, BLOCK_SIDE as f32].component_mul(&vol_scale);

    BoundBox::from_position_dims(block_pos, block_dims)
}

// todo redo
pub fn get_block_data(
    volume: &[u8],
    size: Vector3<usize>,
    block_start: Point3<usize>,
) -> [f32; BLOCK_DATA_LEN] {
    let mut data = [0.0; BLOCK_DATA_LEN]; // todo background value
    let mut ptr = 0;
    for off_x in 0..BLOCK_SIDE {
        for off_y in 0..BLOCK_SIDE {
            for off_z in 0..BLOCK_SIDE {
                let pos = block_start + vector![off_x, off_y, off_z];
                if pos.x < size.x && pos.y < size.y && pos.z < size.z {
                    let index = get_3d_index(size, pos);
                    let value = volume[index];
                    data[ptr] = value as f32;
                }
                ptr += 1;
            }
        }
    }
    data
}

fn get_3d_index(size: Vector3<usize>, pos: Point3<usize>) -> usize {
    pos.z + pos.y * size.z + pos.x * size.y * size.z
}

#[cfg(test)]
mod test {

    use nalgebra::{point, vector};

    use crate::test_helpers::skull_volume;

    use super::*;

    // Copy of block indexing for testing (no constants)
    fn get_indexes(
        block_side: usize,
        block_overlap: usize,
        blocks_size: Vector3<usize>,
        x: usize,
        y: usize,
        z: usize,
    ) -> (usize, usize) {
        let jump_per_block = block_side - block_overlap;
        //assert_ne!(jump_per_block, 0);
        let block_offset = (z % jump_per_block)
            + (y % jump_per_block) * block_side
            + (x % jump_per_block) * block_side * block_side;
        let block_index = (z / jump_per_block)
            + (y / jump_per_block) * blocks_size.z
            + (x / jump_per_block) * blocks_size.y * blocks_size.z;
        (block_index, block_offset)
    }

    // #[test]
    // #[ignore]
    // fn construction() {
    //     // Assumes BLOCK_SIDE == 3
    //     let mut data = [0.0; 3 * 3 * 3];
    //     data[2] = 1.9;
    //     data[9] = 1.8;
    //     data[20] = 0.0;
    //     let bbox = BoundBox::new(point![0.0, 0.0, 0.0], point![1.0, 1.0, 1.0]);
    //     let block = Block::from_data(data, bbox);

    //     assert_eq!(block.value_range.limits(), (0.0, 1.9));
    // }

    #[test]
    fn block_indexing_3() {
        let block_size = vector![5, 5, 5];

        assert_eq!(get_indexes(3, 1, block_size, 0, 0, 0), (0, 0));
        assert_eq!(get_indexes(3, 1, block_size, 0, 0, 1), (0, 1));
        assert_eq!(get_indexes(3, 1, block_size, 0, 0, 2), (1, 0));
    }

    #[test]
    fn block_indexing_7() {
        let block_size = vector![5, 5, 5];

        assert_eq!(get_indexes(7, 1, block_size, 0, 0, 0), (0, 0));
        assert_eq!(get_indexes(7, 1, block_size, 0, 0, 1), (0, 1));
        assert_eq!(get_indexes(7, 1, block_size, 0, 0, 2), (0, 2));
        assert_eq!(get_indexes(7, 1, block_size, 0, 0, 6), (1, 0));
        assert_eq!(get_indexes(7, 1, block_size, 0, 0, 7), (1, 1));
        assert_eq!(
            get_indexes(7, 1, block_size, 0, 7, 7),
            (block_size.z + 1, 7 + 1)
        );

        assert_eq!(
            get_indexes(7, 1, block_size, 4, 3, 2),
            (0, 7 * 7 * 4 + 7 * 3 + 2)
        );
    }

    #[test]
    fn getting_block_data() {
        // Assumes BLOCK_SIDE == 4
        let v: Vec<u8> = (0..=255).into_iter().cycle().take(10 * 10 * 10).collect();
        let vol_data = &v[..];

        let size = vector![10, 10, 10];

        let c = get_block_data(vol_data, size, point![0, 0, 0]);
        assert_eq!(c[0..4], [0.0, 1.0, 2.0, 3.0]);
        assert_eq!(c[4..8], [10.0, 11.0, 12.0, 13.0]);
        assert_eq!(c[8], 20.0);

        assert_eq!(c[(4 * 4)..((4 * 4) + 4)], [100.0, 101.0, 102.0, 103.0]);
    }

    #[test]
    fn block_order() {
        // Assumes BLOCK_SIDE == 4
        let volume: BlockVolume = skull_volume();
    }
}
