use nalgebra::{vector, Vector3};

use super::{
    vol_builder::{color, BuildVolume, RGBA},
    Volume, VolumeBuilder,
};

const BLOCK_SIDE: usize = 3;
const BLOCK_OVERLAP: usize = 1;
const BLOCK_DATA_LEN: usize = BLOCK_SIDE.pow(3);

pub struct BlockVolume {
    data_size: Vector3<usize>,
    block_size: Vector3<usize>,
    border: u32,
    scale: Vector3<f32>,    // shape of voxels
    vol_dims: Vector3<f32>, // size * scale = resulting size of bounding box ; max of bounding box
    data: Vec<Block>,
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
        let block_offset = (z % jump_per_block)
            + (y % jump_per_block) * jump_per_block
            + (x % jump_per_block) * jump_per_block * jump_per_block;
        let block_index = (z / jump_per_block)
            + (y / jump_per_block) * self.block_size.z
            + (x / jump_per_block) * self.block_size.y * self.block_size.z;
        (block_index, block_offset)
    }

    // get voxel
    fn get_3d_data(&self, x: usize, y: usize, z: usize) -> RGBA {
        let block_index = self.get_block_index(x, y, z);
        let block_offset = self.get_block_offset(x, y, z);
        let res = self.data[block_index].data[block_offset];
        // println!(
        //     "> ({},{},{}) -> block {} offset {} = {}",
        //     x, y, z, block_index, block_offset, res
        // );
        res
    }

    // block data, base offset
    fn get_block_data(&self, pos: Vector3<f32>) -> [RGBA; 8] {
        let x = pos.x as usize;
        let y = pos.y as usize;
        let z = pos.z as usize;

        let (block_index, block_offset) = self.get_indexes(x, y, z);

        let block = &self.data[block_index];

        [
            block.data[block_offset],
            block.data[block_offset + 1],
            block.data[block_offset + BLOCK_SIDE],
            block.data[block_offset + BLOCK_SIDE + 1],
            block.data[block_offset + BLOCK_SIDE * BLOCK_SIDE],
            block.data[block_offset + BLOCK_SIDE * BLOCK_SIDE + 1],
            block.data[block_offset + BLOCK_SIDE * BLOCK_SIDE + BLOCK_SIDE],
            block.data[block_offset + BLOCK_SIDE * BLOCK_SIDE + BLOCK_SIDE + 1],
        ]

        // println!(
        //     "> ({},{},{}) -> block {} offset {}",
        //     x, y, z, block_index, block_offset
        // );
    }
}

pub struct Block {
    data: [RGBA; BLOCK_DATA_LEN],
}

impl Block {
    pub fn from_data(data: [RGBA; BLOCK_DATA_LEN]) -> Block {
        Block { data }
    }
}

impl Volume for BlockVolume {
    fn get_size(&self) -> Vector3<usize> {
        self.data_size
    }

    fn get_dims(&self) -> Vector3<f32> {
        self.vol_dims
    }

    fn sample_at(&self, pos: Vector3<f32>) -> RGBA {
        let data = self.get_block_data(pos);

        let x_t = pos.x.fract();
        let y_t = pos.y.fract();
        let z_t = pos.z.fract();

        let [c000, c001, c010, c011, c100, c101, c110, c111] = data;

        let inv_x_t = 1.0 - x_t;
        let c00 = c000 * inv_x_t + c100 * x_t; // todo zmena poradi trilin. - nejdriv steny co jsou u sebe adresove
        let c01 = c001 * inv_x_t + c101 * x_t;
        let c10 = c010 * inv_x_t + c110 * x_t;
        let c11 = c011 * inv_x_t + c111 * x_t;

        let inv_y_t = 1.0 - y_t;
        let c0 = c00 * inv_y_t + c10 * y_t;
        let c1 = c01 * inv_y_t + c11 * y_t;

        c0 * (1.0 - z_t) + c1 * z_t
    }

    fn is_in(&self, pos: &Vector3<f32>) -> bool {
        self.vol_dims.x > pos.x
            && self.vol_dims.y > pos.y
            && self.vol_dims.z > pos.z
            && pos.x > 0.0
            && pos.y > 0.0
            && pos.z > 0.0
    }

    fn get_data(&self, x: usize, y: usize, z: usize) -> RGBA {
        self.get_3d_data(x, y, z)
    }
}

pub fn get_block(builder: &VolumeBuilder, x: usize, y: usize, z: usize) -> Block {
    let mut v = [color::zero(); BLOCK_DATA_LEN]; // todo push
    let mut ptr = 0;
    for off_x in 0..BLOCK_SIDE {
        for off_y in 0..BLOCK_SIDE {
            for off_z in 0..BLOCK_SIDE {
                if x + off_x >= builder.size.x
                    || y + off_y >= builder.size.y
                    || z + off_z >= builder.size.z
                {
                    v[ptr] = color::zero(); // todo inefficient
                } else {
                    let value = builder.get_data(x + off_x, y + off_y, z + off_z);
                    v[ptr] = value;
                }
                ptr += 1;
            }
        }
    }
    Block::from_data(v)
}

impl BuildVolume for BlockVolume {
    fn build(builder: VolumeBuilder) -> Self {
        let vol_dims = (builder.size - vector![1, 1, 1]) // side length is n-1 times the point
            .cast::<f32>();
        let vol_dims = (vol_dims - vector![0.1, 0.1, 0.1]).component_mul(&builder.scale); // todo workaround

        let mut data = vec![];

        let step_size = BLOCK_SIDE - BLOCK_OVERLAP;

        for x in (0..builder.size.x).step_by(step_size) {
            for y in (0..builder.size.y).step_by(step_size) {
                for z in (0..builder.size.z).step_by(step_size) {
                    let block = get_block(&builder, x, y, z);
                    data.push(block);
                }
            }
        }

        let block_size = vector![
            (builder.size.x / step_size) as usize,
            (builder.size.y / step_size) as usize,
            (builder.size.z / step_size) as usize
        ];

        println!(
            "Built {} blocks of dims {} {}",
            data.len(),
            BLOCK_SIDE,
            BLOCK_DATA_LEN
        );

        BlockVolume {
            data_size: builder.size,
            block_size,
            border: builder.border,
            scale: builder.scale,
            vol_dims,
            data,
        }
    }
}

#[cfg(test)]
mod test {

    use crate::{vol_reader, volumetric::LinearVolume};

    use super::*;

    fn cube_volume<V>() -> V
    where
        V: Volume + BuildVolume,
    {
        VolumeBuilder::white_vol().build()
    }

    #[test]
    fn matches_with_linear() {
        let linear: LinearVolume = cube_volume();
        let block: BlockVolume = cube_volume();

        let vol_size = linear.get_size();

        for x in 0..vol_size.x {
            for y in 0..vol_size.y {
                for z in 0..vol_size.z {
                    let lin_data = linear.get_data(x, y, z);
                    let bl_data = block.get_data(x, y, z);
                    println!("check {} {} {} -- {} vs {}", x, y, z, lin_data, bl_data);

                    let dif = (lin_data - bl_data).abs();

                    if dif.iter().any(|&f| f > f32::EPSILON) {
                        println!("failed");
                        assert!(false);
                    }
                }
            }
        }
    }

    #[test] // #[ignore]
    fn matches_with_linear_skull() {
        let builder = vol_reader::from_file("Skull.vol").expect("skull error");
        let linear: LinearVolume = builder.build();
        let builder = vol_reader::from_file("Skull.vol").expect("skull error");
        let block: BlockVolume = builder.build();

        let vol_size = linear.get_size();
        println!("Volsize {}", vol_size);

        for x in 0..vol_size.x {
            for y in 0..vol_size.y {
                for z in 0..vol_size.z {
                    let lin = linear.get_data(x, y, z);
                    let bl = block.get_data(x, y, z);
                    let dif = (lin - bl).abs();

                    let matching = dif.iter().all(|&f| f < f32::EPSILON);
                    assert!(matching);
                }
            }
        }
    }
}
