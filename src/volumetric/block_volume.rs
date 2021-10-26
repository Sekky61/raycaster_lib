use nalgebra::{vector, Vector3};

use crate::ray::Ray;

use super::{vol_builder::BuildVolume, Volume, VolumeBuilder};

pub struct BlockVolume {
    data_size: Vector3<usize>,
    block_size: Vector3<usize>,
    border: u32,
    scale: Vector3<f32>,    // shape of voxels
    vol_dims: Vector3<f32>, // size * scale = resulting size of bounding box ; max of bounding box
    data: Vec<Block>,
}

impl BlockVolume {
    fn get_block_offset(&self, x: usize, y: usize, z: usize) -> usize {
        (z % BLOCK_SIDE)
            + (y % BLOCK_SIDE) * BLOCK_SIDE
            + (x % BLOCK_SIDE) * BLOCK_SIDE * BLOCK_SIDE
    }

    // return: block, offset
    fn get_block_index(&self, x: usize, y: usize, z: usize) -> usize {
        (z / BLOCK_SIDE)
            + (y / BLOCK_SIDE) * self.block_size.z
            + (x / BLOCK_SIDE) * self.block_size.y * self.block_size.z
    }

    fn get_3d_data(&self, x: usize, y: usize, z: usize) -> f32 {
        let block_index = self.get_block_index(x, y, z);
        let block_offset = self.get_block_offset(x, y, z);
        // println!(
        //     "> ({},{},{}) -> block {} offset {}",
        //     x, y, z, block_index, block_offset
        // );
        self.data[block_index].data[block_offset]
    }
}

const BLOCK_SIDE: usize = 3;
const BLOCK_DATA_LEN: usize = BLOCK_SIDE.pow(3);

pub struct Block {
    data: [f32; BLOCK_DATA_LEN],
}

impl Block {
    pub fn from_data(data: [f32; BLOCK_DATA_LEN]) -> Block {
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

    fn sample_at(&self, pos: &Vector3<f32>) -> f32 {
        let x_low = pos.x as usize;
        let y_low = pos.y as usize;
        let z_low = pos.z as usize;

        let x_high = x_low + 1;
        let y_high = y_low + 1;
        let z_high = z_low + 1;

        let x_t = pos.x.fract();
        let y_t = pos.y.fract();
        let z_t = pos.z.fract();

        let c000 = self.get_3d_data(x_low, y_low, z_low);
        let c001 = self.get_3d_data(x_low, y_low, z_high);
        let c010 = self.get_3d_data(x_low, y_high, z_low);
        let c011 = self.get_3d_data(x_low, y_high, z_high);
        let c100 = self.get_3d_data(x_high, y_low, z_low);
        let c101 = self.get_3d_data(x_high, y_low, z_high);
        let c110 = self.get_3d_data(x_high, y_high, z_low);
        let c111 = self.get_3d_data(x_high, y_high, z_high);

        let inv_x_t = 1.0 - x_t;
        let c00 = c000 * inv_x_t + c100 * x_t;
        let c01 = c001 * inv_x_t + c101 * x_t;
        let c10 = c010 * inv_x_t + c110 * x_t;
        let c11 = c011 * inv_x_t + c111 * x_t;

        let inv_y_t = 1.0 - y_t;
        let c0 = c00 * inv_y_t + c10 * y_t;
        let c1 = c01 * inv_y_t + c11 * y_t;

        c0 * (1.0 - z_t) + c1 * z_t
    }

    fn is_in(&self, pos: Vector3<f32>) -> bool {
        self.vol_dims.x > pos.x
            && self.vol_dims.y > pos.y
            && self.vol_dims.z > pos.z
            && pos.x > 0.0
            && pos.y > 0.0
            && pos.z > 0.0
    }

    fn get_data(&self, x: usize, y: usize, z: usize) -> f32 {
        self.get_3d_data(x, y, z)
    }
}

pub fn get_block(builder: &VolumeBuilder, x: usize, y: usize, z: usize) -> Block {
    let mut v = [0.0; BLOCK_DATA_LEN];
    let mut ptr = 0;
    for off_x in 0..BLOCK_SIDE {
        for off_y in 0..BLOCK_SIDE {
            for off_z in 0..BLOCK_SIDE {
                let value = builder.get_data(x + off_x, y + off_y, z + off_z);
                v[ptr] = value;
                ptr += 1;
            }
        }
    }
    Block { data: v }
}

impl BuildVolume for BlockVolume {
    fn build(builder: VolumeBuilder) -> Self {
        let vol_dims = (builder.size - vector![1, 1, 1]) // side length is n-1 times the point
            .cast::<f32>()
            .component_mul(&builder.scale);

        let mut data = vec![];

        for x in (0..builder.size.x).step_by(3) {
            for y in (0..builder.size.y).step_by(3) {
                for z in (0..builder.size.z).step_by(3) {
                    let block = get_block(&builder, x, y, z);
                    data.push(block);
                }
            }
        }

        let block_size = vector![
            (builder.size.x as f32 / BLOCK_SIDE as f32).ceil() as usize,
            (builder.size.y as f32 / BLOCK_SIDE as f32).ceil() as usize,
            (builder.size.z as f32 / BLOCK_SIDE as f32).ceil() as usize
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

    use nalgebra::vector;

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
                    assert!(
                        (linear.get_data(x, y, z) - block.get_data(x, y, z)).abs() < f32::EPSILON
                    );
                }
            }
        }
    }

    #[test] // #[ignore]
    fn matches_with_skull() {
        let builder = vol_reader::from_file("Skull.vol").expect("skull error");
        let linear: LinearVolume = builder.build();
        let builder = vol_reader::from_file("Skull.vol").expect("skull error");
        let block: BlockVolume = builder.build();

        let vol_size = linear.get_size();
        println!("Volsize {}", vol_size);

        for x in 0..vol_size.x {
            for y in 0..vol_size.y {
                for z in 0..vol_size.z {
                    assert!(
                        (linear.get_data(x, y, z) - block.get_data(x, y, z)).abs() < f32::EPSILON
                    );
                }
            }
        }
    }
}
