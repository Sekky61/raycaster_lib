use nalgebra::{point, vector, Point3, Vector3};

use crate::common::{BoundBox, ValueRange};

use super::{
    vol_builder::{BuildVolume, VolumeMetadata},
    Volume, TF,
};

const BLOCK_SIDE: usize = 3;
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
    // block 3d index -> 2d index
    fn get_block_offset(&self, x: usize, y: usize, z: usize) -> usize {
        let jump_per_block = BLOCK_SIDE - BLOCK_OVERLAP; // todo bug here for low coords
        (z % jump_per_block)
            + (y % jump_per_block) * BLOCK_SIDE
            + (x % jump_per_block) * BLOCK_SIDE * BLOCK_SIDE
    }

    // return: voxel 3d index -> block 2d index
    fn get_block_index(&self, x: usize, y: usize, z: usize) -> usize {
        let jump_per_block = BLOCK_SIDE - BLOCK_OVERLAP;
        (z / jump_per_block)
            + (y / jump_per_block) * self.block_size.z
            + (x / jump_per_block) * self.block_size.y * self.block_size.z
    }

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
        let block_size = size.map(|v| (v / step_size) as usize);

        let slice = &data.get_slice().ok_or("No data in datasource")?[offset..];

        for x in (0..size.x).step_by(step_size) {
            for y in (0..size.y).step_by(step_size) {
                for z in (0..size.z).step_by(step_size) {
                    let block_start = point![x, y, z];
                    let block_data = get_block_data(slice, size, block_start);
                    let block_bound_box = get_bound_box(position, scale, block_size, block_start);
                    let block = Block::from_data(block_data, block_bound_box);
                    blocks.push(block);
                }
            }
        }

        println!(
            "Built {} blocks of dims {} {}",
            blocks.len(),
            BLOCK_SIDE,
            BLOCK_DATA_LEN
        );

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
    block_size: Vector3<usize>,
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
