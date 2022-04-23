use nalgebra::{point, vector, Point3, Vector3};

use crate::{
    common::{blockify, tf_visible_range, BoundBox},
    TF,
};

use super::{
    float_block::FloatBlock,
    vol_builder::{BuildVolume, VolumeMetadata},
    volume::Blocked,
    Volume,
};

// Default overlap == 1
pub struct FloatBlockVolume {
    block_side: usize,
    bound_box: BoundBox,
    data_size: Vector3<usize>,
    pub empty_blocks: Vec<bool>, // todo use empty index, but first remove generics from emptyindex
    block_size: Vector3<usize>,  // Number of blocks in structure (.data)
    pub data: Vec<FloatBlock>,
    tf: TF,
}

impl FloatBlockVolume {
    // returns (block index, block offset)
    fn get_indexes(&self, x: usize, y: usize, z: usize) -> (usize, usize) {
        let jump_per_block = self.block_side - 1; // implicit block overlap of 1
        let block_offset = (z % jump_per_block)
            + (y % jump_per_block) * self.block_side
            + (x % jump_per_block) * self.block_side * self.block_side;
        let block_index = (z / jump_per_block)
            + (y / jump_per_block) * self.block_size.z
            + (x / jump_per_block) * self.block_size.y * self.block_size.z;
        (block_index, block_offset)
    }

    // get voxel
    // todo make unchecked version
    fn get_3d_data(&self, x: usize, y: usize, z: usize) -> Option<f32> {
        let (block_index, block_offset) = self.get_indexes(x, y, z);
        match self.data.get(block_index) {
            Some(b) => b.data.get(block_offset).copied(),
            None => None,
        }
    }

    pub fn build_empty(blocks: &[FloatBlock], tf: TF) -> Vec<bool> {
        let mut v = Vec::with_capacity(blocks.len());
        let vis_ranges = tf_visible_range(tf);

        for block in blocks {
            let visible = vis_ranges.iter().any(|r| r.intersects(&block.value_range));
            v.push(!visible);
        }
        v
    }
}

impl Blocked for FloatBlockVolume {
    type BlockType = FloatBlock;

    fn get_blocks(&self) -> &[Self::BlockType] {
        &self.data
    }

    fn get_empty_blocks(&self) -> &[bool] {
        &self.empty_blocks
    }
}

impl Volume for FloatBlockVolume {
    fn get_size(&self) -> Vector3<usize> {
        self.data_size
    }

    fn sample_at(&self, pos: Point3<f32>) -> f32 {
        let x = pos.x as usize;
        let y = pos.y as usize;
        let z = pos.z as usize;

        let x_t = pos.x.fract();
        let y_t = pos.y.fract();
        let z_t = pos.z.fract();

        let (block_index, block_offset) = self.get_indexes(x, y, z);

        let block = &self.data[block_index];
        let first_index = block_offset;
        let second_index = block_offset + self.block_side * self.block_side;

        // first plane
        // c000, c001, c010, c011
        let mut x_low_vec = block.get_block_data_half(first_index);

        // second plane
        // c100, c101, c110, c111
        let mut x_hi_vec = block.get_block_data_half(second_index);

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

    fn get_data(&self, x: usize, y: usize, z: usize) -> Option<f32> {
        self.get_3d_data(x, y, z)
    }

    fn get_tf(&self) -> TF {
        self.tf
    }

    fn get_bound_box(&self) -> BoundBox {
        self.bound_box
    }

    fn get_scale(&self) -> Vector3<f32> {
        vector![1.0, 1.0, 1.0] // todo
    }

    fn set_tf(&mut self, tf: TF) {
        self.tf = tf;
        self.empty_blocks = FloatBlockVolume::build_empty(&self.data, self.tf);
    }

    fn get_name() -> &'static str {
        "FloatBlockVolume"
    }

    fn is_empty(&self, pos: Point3<f32>) -> bool {
        let x = pos.x as usize;
        let y = pos.y as usize;
        let z = pos.z as usize;
        let (block_index, _) = self.get_indexes(x, y, z);
        self.empty_blocks[block_index]
    }
}

impl BuildVolume<u8> for FloatBlockVolume {
    fn build(metadata: VolumeMetadata<u8>) -> Result<FloatBlockVolume, &'static str> {
        let position = metadata.position.unwrap_or_else(|| point![0.0, 0.0, 0.0]);
        let size = metadata.size.ok_or("No size")?;
        let scale = metadata.scale.ok_or("No scale")?;
        let data = metadata.data.ok_or("No data")?;
        let tf = metadata.tf.ok_or("No transfer function")?;
        let block_side = metadata.block_side.ok_or("No block side")?;

        let vol_dims = (size - vector![1, 1, 1]) // side length is n-1 times the point
            .cast::<f32>();
        let vol_dims = (vol_dims).component_mul(&scale); // todo workaround

        let bound_box = BoundBox::from_position_dims(position, vol_dims);

        let step_size = block_side - 1;
        let block_size = blockify(size, block_side, 1);

        let mut blocks = Vec::with_capacity(block_size.product());

        let slice = data.get_slice();

        for x in 0..block_size.x {
            for y in 0..block_size.y {
                for z in 0..block_size.z {
                    let block_start = step_size * point![x, y, z];
                    let block_data = get_block_data(slice, size, block_start, block_side);
                    let block_bound_box = get_bound_box(position, scale, block_start, block_side);
                    let block =
                        FloatBlock::from_data(block_data, block_bound_box, scale, block_side, tf);
                    blocks.push(block);
                }
            }
        }

        let empty_blocks = FloatBlockVolume::build_empty(&blocks, tf);

        println!(
            "Built {} blocks of dims {} blocks ({},{},{}) -> ({},{},{})",
            blocks.len(),
            block_side,
            size.x,
            size.y,
            size.z,
            block_size.x,
            block_size.y,
            block_size.z,
        );

        Ok(FloatBlockVolume {
            bound_box,
            data_size: size,
            block_size,
            data: blocks,
            tf,
            block_side,
            empty_blocks,
        })
    }
}

fn get_bound_box(
    vol_position: Point3<f32>,
    vol_scale: Vector3<f32>,
    block_start: Point3<usize>,
    block_side: usize,
) -> BoundBox {
    let block_lower = vector![
        block_start.x as f32,
        block_start.y as f32,
        block_start.z as f32
    ];

    let block_dims = vector![block_side - 1, block_side - 1, block_side - 1].cast::<f32>();
    let block_dims = block_dims.component_mul(&vol_scale); // - vector![0.01, 0.01, 0.01]; // todo workaround

    let block_pos = vol_position + block_lower.component_mul(&vol_scale);

    BoundBox::from_position_dims(block_pos, block_dims)
}

// todo redo
pub fn get_block_data(
    volume: &[u8],
    size: Vector3<usize>,
    block_start: Point3<usize>,
    side: usize,
) -> Vec<f32> {
    let mut data = Vec::with_capacity(side * side * side); // todo background value
    for off_x in 0..side {
        for off_y in 0..side {
            for off_z in 0..side {
                let pos = block_start + vector![off_x, off_y, off_z];
                if pos.x < size.x && pos.y < size.y && pos.z < size.z {
                    let index = get_3d_index(size, pos);
                    let value = volume[index];
                    data.push(value as f32);
                } else {
                    data.push(0.0);
                }
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

    #[test]
    fn build_empty() {
        let tf = |v: f32| vector![1.0, 1.0, 1.0, v];
        let block1 =
            FloatBlock::from_data(vec![0.0], BoundBox::empty(), vector![1.0, 1.0, 1.0], 1, tf);
        let block2 =
            FloatBlock::from_data(vec![1.0], BoundBox::empty(), vector![1.0, 1.0, 1.0], 1, tf);
        let block3 =
            FloatBlock::from_data(vec![2.0], BoundBox::empty(), vector![1.0, 1.0, 1.0], 1, tf);
        let blocks = &[block1, block2, block3];

        let empty = FloatBlockVolume::build_empty(blocks, tf);

        assert_eq!(empty.len(), 3);
        assert!(empty[0]);
        assert!(!empty[1]);
        assert!(!empty[2]);
    }

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
        let block_side = 4;

        let v: Vec<u8> = (0..=255).into_iter().cycle().take(10 * 10 * 10).collect();
        let vol_data = &v[..];

        let size = vector![10, 10, 10];

        let c = get_block_data(vol_data, size, point![0, 0, 0], block_side);
        assert_eq!(c[0..4], [0.0, 1.0, 2.0, 3.0]);
        assert_eq!(c[4..8], [10.0, 11.0, 12.0, 13.0]);
        assert_eq!(c[8], 20.0);

        assert_eq!(c[(4 * 4)..((4 * 4) + 4)], [100.0, 101.0, 102.0, 103.0]);
    }
}
