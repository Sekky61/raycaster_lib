use super::VolumeBuilder;
use nalgebra::Vector3;

pub trait Volume {
    // get scaled size
    fn get_dims(&self) -> Vector3<f32>;

    // trilinear interpolation sample, zero if outside
    fn sample_at(&self, pos: Vector3<f32>) -> f32;

    // position is inside volume
    fn is_in(&self, pos: Vector3<f32>) -> bool;
}

pub struct LinearVolume {
    size: Vector3<usize>,
    border: u32,
    scale: Vector3<f32>,    // shape of voxels
    vol_dims: Vector3<f32>, // size * scale = resulting size of bounding box
    data: Vec<u8>,
}

impl std::fmt::Debug for LinearVolume {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Volume")
            .field("size", &self.size)
            .field("border", &self.border)
            .field("scale", &self.scale)
            .field("vol_dims", &self.vol_dims)
            .field("data len ", &self.data.len())
            .finish()
    }
}

impl LinearVolume {
    fn get_3d_index(&self, x: usize, y: usize, z: usize) -> usize {
        z + y * self.size.z + x * self.size.y * self.size.z
    }

    fn get_3d_data(&self, x: usize, y: usize, z: usize) -> u8 {
        //println!("Getting {} {} {}", x, y, z);
        let val = self.data.get(self.get_3d_index(x, y, z));
        match val {
            Some(&v) => v,
            None => 0,
        }
    }
}

impl Volume for LinearVolume {
    fn get_dims(&self) -> Vector3<f32> {
        self.vol_dims
    }

    fn sample_at(&self, pos: Vector3<f32>) -> f32 {
        let x_low = pos[0].floor() as usize;
        let y_low = pos[1].floor() as usize;
        let z_low = pos[2].floor() as usize;

        let x_high = x_low + 1;
        let y_high = y_low + 1;
        let z_high = z_low + 1;

        let x_t = pos.x.fract();
        let y_t = pos.y.fract();
        let z_t = pos.z.fract();

        let c000 = self.get_3d_data(x_low, y_low, z_low) as f32;
        let c001 = self.get_3d_data(x_low, y_low, z_high) as f32;
        let c010 = self.get_3d_data(x_low, y_high, z_low) as f32;
        let c011 = self.get_3d_data(x_low, y_high, z_high) as f32;
        let c100 = self.get_3d_data(x_high, y_low, z_low) as f32;
        let c101 = self.get_3d_data(x_high, y_low, z_high) as f32;
        let c110 = self.get_3d_data(x_high, y_high, z_low) as f32;
        let c111 = self.get_3d_data(x_high, y_high, z_high) as f32;

        let c00 = c000 * (1.0 - x_t) + c100 * x_t;
        let c01 = c001 * (1.0 - x_t) + c101 * x_t;
        let c10 = c010 * (1.0 - x_t) + c110 * x_t;
        let c11 = c011 * (1.0 - x_t) + c111 * x_t;

        let c0 = c00 * (1.0 - y_t) + c10 * y_t;
        let c1 = c01 * (1.0 - y_t) + c11 * y_t;

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
}

impl From<VolumeBuilder> for LinearVolume {
    fn from(builder: VolumeBuilder) -> LinearVolume {
        let vol_dims = builder.size.cast::<f32>().component_mul(&builder.scale);
        LinearVolume {
            size: builder.size,
            border: builder.border,
            scale: builder.scale,
            vol_dims,
            data: builder.data,
        }
    }
}

// fn get_3d_blocks(&self, x: usize, y: usize, z: usize) -> u8 {
//     //println!("Getting {} {} {}", x, y, z);
//     let block = self.blocks.get_block(x, y, z);
//     let block = match block {
//         Some(b) => b,
//         None => return 0,
//     };
//     block.get_data(x % BLOCK_SIZE, y % BLOCK_SIZE, z % BLOCK_SIZE)
// }

/*
    pub fn blocks_from_frames(&mut self) {
        let num_blocks_x = (self.x as f32 / BLOCK_SIZE as f32).ceil() as usize;
        let num_blocks_y = (self.y as f32 / BLOCK_SIZE as f32).ceil() as usize;
        let num_blocks_z = (self.z as f32 / BLOCK_SIZE as f32).ceil() as usize;

        let num_of_blocks = num_blocks_x + num_blocks_y + num_blocks_z;

        let mut blocks: Vec<Block> = vec![];

        for b_z in 0..num_blocks_z {
            for b_y in 0..num_blocks_y {
                for b_x in 0..num_blocks_x {
                    let mut block = Block::new();

                    let x_coord = b_x * BLOCK_SIZE;
                    let y_coord = b_y * BLOCK_SIZE;

                    for z in 0..BLOCK_SIZE {
                        let ind = b_z * BLOCK_SIZE + z;
                        let plane = self.data.get(ind);
                        match plane {
                            Some(fr) => {
                                let mut slic = fr.get_square_cutout(x_coord, y_coord, BLOCK_SIZE);
                                block.data.append(&mut slic);
                            }
                            None => {
                                let mut slic = vec![0; BLOCK_SIZE * BLOCK_SIZE];
                                block.data.append(&mut slic);
                            }
                        }
                    }

                    blocks.push(block);
                }
            }
        }

        self.block_dim = (num_blocks_x, num_blocks_y, num_blocks_z);
        self.blocks = blocks;
    }
*/
