use memmap::Mmap;
use nalgebra::{point, vector, Matrix4, Point3, Vector3};

use crate::{
    common::{blockify, tf_visible_range, BoundBox, ValueRange},
    volumetric::DataSource,
    TF,
};

use super::{
    vol_builder::{BuildVolume, VolumeMetadata},
    Volume,
};

pub struct StreamBlock {
    pub block_side: usize,
    pub value_range: ValueRange,
    pub bound_box: BoundBox,
    pub transform: Matrix4<f32>,
    pub data: *const u8,
}

impl StreamBlock {
    /// # Safety
    ///
    /// data has to be pointer into the beginning of memory mapped file
    pub unsafe fn new(
        block_side: usize,
        bound_box: BoundBox,
        scale: Vector3<f32>,
        data: *const u8,
    ) -> Self {
        let elements = block_side.pow(3);
        let slice = std::slice::from_raw_parts(data, elements);
        let value_range = ValueRange::from_iter(slice);

        let scale_inv = vector![1.0, 1.0, 1.0].component_div(&scale);
        let lower_vec = point![0.0, 0.0, 0.0] - bound_box.lower; // todo type workaround

        let transform = Matrix4::identity()
            .append_translation(&lower_vec)
            .append_nonuniform_scaling(&scale_inv);

        Self {
            block_side,
            value_range,
            bound_box,
            transform,
            data,
        }
    }

    fn get_block_data_half(&self, start_index: usize) -> [u8; 4] {
        unsafe {
            let ptr = self.data.add(start_index);
            let d0 = ptr.read();

            let ptr = ptr.add(1);
            let d1 = ptr.read();

            let ptr = ptr.add(self.block_side);
            let d2 = ptr.read();

            let ptr = ptr.add(1);
            let d3 = ptr.read();

            [d0, d1, d2, d3]
        }
    }
}

// Safety: pointer points to memory mapped file, which lives as long as StreamBlockVolume lives
unsafe impl Send for StreamBlock {}

// Default overlap == 1
pub struct StreamBlockVolume {
    block_side: usize,
    bound_box: BoundBox,
    data_size: Vector3<usize>,
    pub empty_blocks: Vec<bool>,
    block_size: Vector3<usize>, // Number of blocks in structure (.data)
    _data_owner: Mmap,
    pub data: Vec<StreamBlock>,
    tf: TF,
}

impl StreamBlockVolume {
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

    pub fn build_empty(blocks: &[StreamBlock], tf: TF) -> Vec<bool> {
        let mut v = Vec::with_capacity(blocks.len());
        let vis_ranges = tf_visible_range(tf);

        for block in blocks {
            let visible = vis_ranges.iter().any(|r| r.intersects(&block.value_range));
            v.push(!visible);
        }
        v
    }
}

impl Volume for StreamBlockVolume {
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

        let first_data = block.get_block_data_half(first_index);
        let [c000, c001, c010, c011] = first_data;

        let c000 = c000 as f32;
        let c001 = c001 as f32;
        let c010 = c010 as f32;
        let c011 = c011 as f32;

        let inv_z_t = 1.0 - z_t;
        let inv_y_t = 1.0 - y_t;

        // first plane

        let c00 = c000 * inv_z_t + c001 * z_t; // z low
        let c01 = c010 * inv_z_t + c011 * z_t; // z high
        let c0 = c00 * inv_y_t + c01 * y_t; // point on yz plane

        // second plane

        let second_data = block.get_block_data_half(second_index);
        let [c100, c101, c110, c111] = second_data;

        let c100 = c100 as f32;
        let c101 = c101 as f32;
        let c110 = c110 as f32;
        let c111 = c111 as f32;

        let c10 = c100 * inv_z_t + c101 * z_t; // z low
        let c11 = c110 * inv_z_t + c111 * z_t; // z high
        let c1 = c10 * inv_y_t + c11 * y_t; // point on yz plane

        c0 * (1.0 - x_t) + c1 * x_t
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
        self.empty_blocks = StreamBlockVolume::build_empty(&self.data, self.tf);
    }

    fn get_name(&self) -> &str {
        "StreamBlockVolume"
    }
}

impl BuildVolume<u8> for StreamBlockVolume {
    fn build(metadata: VolumeMetadata<u8>) -> Result<StreamBlockVolume, &'static str> {
        let position = metadata.position.unwrap_or_else(|| point![0.0, 0.0, 0.0]);
        let size = metadata.size.ok_or("No size")?;
        let scale = metadata.scale.ok_or("No scale")?;
        let data = metadata.data.ok_or("No data")?;
        let offset = metadata.data_offset.unwrap_or(0);
        let tf = metadata.tf.ok_or("No transfer function")?;
        let block_side = metadata.block_side.ok_or("No block side")?;

        let vol_dims = (size - vector![1, 1, 1]) // side length is n-1 times the point
            .cast::<f32>();
        let vol_dims = (vol_dims).component_mul(&scale); // todo workaround

        let bound_box = BoundBox::from_position_dims(position, vol_dims);

        let step_size = block_side - 1;
        let block_size = blockify(size, block_side, 1);

        let mut blocks = Vec::with_capacity(block_size.product());

        let (data_owner, data_offset) = match data {
            DataSource::Mmap(m) => m.into_inner(),
            _ => return Err("Data not memory mapped"),
        };
        let ptr = unsafe { data_owner.as_ptr().add(offset) }; // todo offset or data_offset?

        for x in 0..block_size.x {
            for y in 0..block_size.y {
                for z in 0..block_size.z {
                    let block_start = step_size * point![x, y, z];
                    let block_bound_box = get_bound_box(position, scale, block_start, block_side);

                    let block_data_offset = get_3d_index(block_size, block_start);
                    let block_data_ptr = unsafe { ptr.add(block_data_offset) };
                    let block = unsafe {
                        StreamBlock::new(block_side, block_bound_box, scale, block_data_ptr)
                    };
                    blocks.push(block);
                }
            }
        }

        let empty_blocks = StreamBlockVolume::build_empty(&blocks, tf);

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

        Ok(StreamBlockVolume {
            bound_box,
            data_size: size,
            block_size,
            data: blocks,
            _data_owner: data_owner,
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
    let block_dims = block_dims.component_mul(&vol_scale) - vector![0.01, 0.01, 0.01]; // todo workaround

    let block_pos = vol_position + block_lower.component_mul(&vol_scale);

    BoundBox::from_position_dims(block_pos, block_dims)
}

fn get_3d_index(size: Vector3<usize>, pos: Point3<usize>) -> usize {
    pos.z + pos.y * size.z + pos.x * size.y * size.z
}
