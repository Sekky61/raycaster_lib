use memmap::Mmap;
use nalgebra::{point, vector, Matrix4, Point3, Vector3, Vector4};

use crate::{
    common::{blockify, tf_visible_range, BoundBox, Ray, ValueRange},
    volumetric::{DataSource, MemoryType},
    TF,
};

use super::{
    vol_builder::{BuildVolume, VolumeMetadata},
    volume::Blocked,
    EmptyIndex, Volume,
};

pub struct Block {
    pub block_side: usize, // todo empty index
    pub value_range: ValueRange,
    pub bound_box: BoundBox,
    pub transform: Matrix4<f32>,
    pub data: *const u8,
    empty_index: EmptyIndex<4>,
}

impl Block {
    /// # Safety
    ///
    /// data has to be pointer into the beginning of memory mapped file
    pub unsafe fn new(
        block_side: usize,
        bound_box: BoundBox,
        scale: Vector3<f32>,
        data: *const u8,
        tf: TF,
    ) -> Self {
        let elements = block_side.pow(3);
        let slice = std::slice::from_raw_parts(data, elements);
        let value_range = ValueRange::from_samples(slice);

        let scale_inv = vector![1.0, 1.0, 1.0].component_div(&scale);
        let lower_vec = point![0.0, 0.0, 0.0] - bound_box.lower; // todo type workaround

        let transform = Matrix4::identity()
            .append_translation(&lower_vec)
            .append_nonuniform_scaling(&scale_inv);

        let mut block = Self {
            block_side,
            value_range,
            bound_box,
            transform,
            data,
            empty_index: EmptyIndex::dummy(),
        };

        //block.empty_index = EmptyIndex::<4>::from_volume_without_tf(&block, tf);
        block
    }

    fn get_block_data_half(&self, start_index: usize) -> Vector4<f32> {
        unsafe {
            let ptr = self.data.add(start_index);
            let d0 = ptr.read();

            let ptr = ptr.add(1);
            let d1 = ptr.read();

            let ptr = ptr.add(self.block_side);
            let d2 = ptr.read();

            let ptr = ptr.add(1);
            let d3 = ptr.read();

            vector![d0 as f32, d1 as f32, d2 as f32, d3 as f32]
        }
    }

    fn get_3d_index(&self, x: usize, y: usize, z: usize) -> usize {
        z + y * self.block_side + x * self.block_side * self.block_side
    }
}

// Safety: pointer points to memory mapped file, which lives as long as BlockVolume lives
unsafe impl Send for Block {}

impl Volume for Block {
    // A more optimal specialization
    fn transform_ray(&self, ray: &Ray) -> Option<(Ray, f32)> {
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

    fn set_tf(&mut self, tf: TF) {
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
        let second_index = first_index + self.block_side * self.block_side;

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
        if index >= self.block_side * self.block_side * self.block_side {
            return None;
        }
        let sample = unsafe { self.data.add(index).read() };
        Some(sample as f32)
    }

    fn get_name() -> &'static str {
        "Block"
    }

    fn is_empty(&self, pos: Point3<f32>) -> bool {
        self.empty_index.is_empty(pos)
    }
}

// Default overlap == 1
pub struct BlockVolume {
    block_side: usize,
    bound_box: BoundBox,
    data_size: Vector3<usize>,
    pub empty_blocks: Vec<bool>,
    block_size: Vector3<usize>, // Number of blocks in structure (.data)
    _data_owner: DataSource<u8>,
    pub data: Vec<Block>,
    tf: TF,
}

unsafe impl Sync for BlockVolume {}

impl BlockVolume {
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
    fn get_3d_data(&self, x: usize, y: usize, z: usize) -> Option<u8> {
        let (block_index, block_offset) = self.get_indexes(x, y, z);
        match self.data.get(block_index) {
            Some(b) => {
                if block_offset < self.block_side.pow(3) {
                    let val = unsafe { std::ptr::read(b.data.add(block_offset)) };
                    Some(val)
                } else {
                    None
                }
            }
            None => None,
        }
    }

    /// True == block is empty
    pub fn build_empty(blocks: &[Block], tf: TF) -> Vec<bool> {
        let mut v = Vec::with_capacity(blocks.len());
        let vis_ranges = tf_visible_range(tf);

        for block in blocks {
            let visible = vis_ranges.iter().any(|r| r.intersects(&block.value_range));
            v.push(!visible);
        }
        v
    }
}

impl Blocked for BlockVolume {
    type BlockType = Block;

    fn get_blocks(&self) -> &[Self::BlockType] {
        &self.data
    }

    fn get_empty_blocks(&self) -> &[bool] {
        &self.empty_blocks
    }
}

impl Volume for BlockVolume {
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
        let sample = self.get_3d_data(x, y, z); // todo bounds check
        sample.map(|v| v as f32)
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
        self.empty_blocks = BlockVolume::build_empty(&self.data, self.tf);
    }

    fn get_name() -> &'static str {
        "BlockVolume"
    }

    fn is_empty(&self, pos: Point3<f32>) -> bool {
        false // todo delegate to block
    }
}

impl BuildVolume<u8> for BlockVolume {
    fn build(metadata: VolumeMetadata<u8>) -> Result<BlockVolume, &'static str> {
        let position = metadata.position.unwrap_or_else(|| point![0.0, 0.0, 0.0]);
        let size = metadata.size.ok_or("No size")?;
        let scale = metadata.scale.ok_or("No scale")?;
        let tf = metadata.tf.ok_or("No transfer function")?;
        let block_side = metadata.block_side.ok_or("No block side")?;
        let data = metadata.data.ok_or("No data")?;
        let memory_type = metadata.memory_type.unwrap_or(MemoryType::Stream);

        let vol_dims = (size - vector![1, 1, 1]) // side length is n-1 times the point
            .cast::<f32>();
        let vol_dims = (vol_dims).component_mul(&scale); // todo workaround

        let bound_box = BoundBox::from_position_dims(position, vol_dims);

        let step_size = block_side - 1;
        let block_size = blockify(size, block_side, 1);

        let mut blocks = Vec::with_capacity(block_size.product());

        let data_desired_form = match memory_type {
            MemoryType::Stream => data,
            MemoryType::Ram => data.to_vec(),
        };

        let ptr = unsafe { data_desired_form.as_ptr() };

        for x in 0..block_size.x {
            for y in 0..block_size.y {
                for z in 0..block_size.z {
                    let block_off = point![x, y, z];
                    let block_start = step_size * block_off;
                    let block_bound_box = get_bound_box(position, scale, block_start, block_side);

                    let block_data_offset = get_3d_index(block_size, block_off) * block_side.pow(3);
                    let block_data_ptr = unsafe { ptr.add(block_data_offset) };
                    let block = unsafe {
                        Block::new(block_side, block_bound_box, scale, block_data_ptr, tf)
                    };
                    blocks.push(block);
                }
            }
        }

        let empty_blocks = BlockVolume::build_empty(&blocks, tf);

        println!(
            "Built {} blocks of dims {} blocks ({},{},{}) blocks ({},{},{}) memory",
            blocks.len(),
            block_side,
            size.x,
            size.y,
            size.z,
            block_size.x,
            block_size.y,
            block_size.z,
        );

        let volume = BlockVolume {
            bound_box,
            data_size: size,
            block_size,
            data: blocks,
            _data_owner: data_desired_form,
            tf,
            block_side,
            empty_blocks,
        };
        Ok(volume)
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
    let block_dims = block_dims.component_mul(&vol_scale);

    let block_pos = vol_position + block_lower.component_mul(&vol_scale);

    BoundBox::from_position_dims(block_pos, block_dims)
}

fn get_3d_index(size: Vector3<usize>, pos: Point3<usize>) -> usize {
    pos.z + pos.y * size.z + pos.x * size.y * size.z
}
