use std::convert::TryInto;
use std::fmt::Display;
use std::path::Path;
use std::{fs::File, io::Read};

use nalgebra::{vector, Vector3};

const BLOCK_SIZE: usize = 16;

struct Block {
    pub data: Vec<u8>,
}

impl Block {
    pub fn new() -> Block {
        Block { data: vec![] }
    }

    pub fn get_data(&self, x: usize, y: usize, z: usize) -> u8 {
        self.data[x + y * BLOCK_SIZE + z * BLOCK_SIZE * BLOCK_SIZE]
    }
}

#[derive(Default)]
pub struct Volume {
    pub x: usize,
    pub y: usize,
    pub z: usize,
    border: u32,
    scale_x: f32,
    scale_y: f32,
    scale_z: f32,
    vol_dims: (f32, f32, f32),
    data: Vec<u8>,
    block_dim: (usize, usize, usize),
    blocks: Vec<Block>,
}

impl std::fmt::Debug for Volume {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Volume")
            .field("x", &self.x)
            .field("y", &self.y)
            .field("z", &self.z)
            .field("border", &self.border)
            .field("scale_x", &self.scale_x)
            .field("scale_y", &self.scale_y)
            .field("scale_z", &self.scale_z)
            .finish()
    }
}

impl Volume {
    pub fn white_vol() -> Volume {
        Volume {
            x: 2,
            y: 2,
            z: 2,
            border: 0,
            scale_x: 1.0,
            scale_y: 1.0,
            scale_z: 1.0,
            vol_dims: (1.0, 1.0, 1.0),
            data: vec![],
            block_dim: (0, 0, 0),
            blocks: vec![],
        }
    }

    pub fn get_dims(&self) -> Vector3<f32> {
        vector![
            self.x as f32 * self.scale_x,
            self.y as f32 * self.scale_y,
            self.z as f32 * self.scale_z
        ]
    }

    // trilinear interpolation sample
    pub fn sample_at(&self, pos: Vector3<f32>) -> f32 {
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

    pub fn from_file<P>(path: P) -> Volume
    where
        P: AsRef<Path>,
    {
        let mut fb = Vec::with_capacity(5_000_000);
        File::open(path)
            .expect("no file skull")
            .read_to_end(&mut fb)
            .expect("cannot read");

        // assert_eq!(parser("(abc)"), Ok(("", "abc")));

        let x_bytes: [u8; 4] = fb.get(0..4).expect("no bytes x").try_into().expect("wrong");
        let x = u32::from_be_bytes(x_bytes);

        let y_bytes: [u8; 4] = fb.get(4..8).expect("no bytes y").try_into().expect("wrong");
        let y = u32::from_be_bytes(y_bytes);

        let z_bytes: [u8; 4] = fb
            .get(8..12)
            .expect("no bytes z")
            .try_into()
            .expect("wrong");
        let z = u32::from_be_bytes(z_bytes);

        let xs_bytes: [u8; 4] = fb
            .get(16..20)
            .expect("no bytes x scale")
            .try_into()
            .expect("wrong");
        let scale_x = f32::from_be_bytes(xs_bytes);

        let ys_bytes: [u8; 4] = fb
            .get(20..24)
            .expect("no bytes y scale")
            .try_into()
            .expect("wrong");
        let scale_y = f32::from_be_bytes(ys_bytes);

        let zs_bytes: [u8; 4] = fb
            .get(24..28)
            .expect("no bytes z scale")
            .try_into()
            .expect("wrong");
        let scale_z = f32::from_be_bytes(zs_bytes);

        let rest = &fb[28..];

        println!(
            "Rest: {} | Rest / 68 = {} | Rest / 256*256 = {}",
            rest.len(),
            rest.len() / 68,
            rest.len() / (256 * 256)
        );

        let data = rest.to_owned();

        let x = x as usize;
        let y = y as usize;
        let z = z as usize;

        let vol_dims = (x as f32 * scale_x, y as f32 * scale_y, z as f32 * scale_z);

        Volume {
            x,
            y,
            z,
            border: 0,
            scale_x,
            scale_y,
            scale_z,
            vol_dims,
            data,
            block_dim: (0, 0, 0),
            blocks: vec![],
        }
    }
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
    pub fn get_3d_index(&self, x: usize, y: usize, z: usize) -> usize {
        z + y * self.z + x * self.y * self.z
    }

    pub fn get_3d_data(&self, x: usize, y: usize, z: usize) -> u8 {
        //println!("Getting {} {} {}", x, y, z);
        let val = self.data.get(self.get_3d_index(x, y, z));
        match val {
            Some(&v) => v,
            None => 0,
        }
    }

    fn get_block(&self, x: usize, y: usize, z: usize) -> Option<&Block> {
        let block_x = (x as f32 / BLOCK_SIZE as f32).ceil() as usize;
        let block_y = (y as f32 / BLOCK_SIZE as f32).ceil() as usize;
        let block_z = (z as f32 / BLOCK_SIZE as f32).ceil() as usize;

        self.blocks.get(
            block_x + block_y * self.block_dim.0 + block_z * self.block_dim.0 * self.block_dim.1,
        )
    }

    pub fn get_3d_blocks(&self, x: usize, y: usize, z: usize) -> u8 {
        //println!("Getting {} {} {}", x, y, z);
        let block = self.get_block(x, y, z);
        let block = match block {
            Some(b) => b,
            None => return 0,
        };
        block.get_data(x % BLOCK_SIZE, y % BLOCK_SIZE, z % BLOCK_SIZE)
    }

    pub fn is_in(&self, pos: Vector3<f32>) -> bool {
        self.vol_dims.0 > pos.x
            && self.vol_dims.1 > pos.y
            && self.vol_dims.2 > pos.z
            && pos.x > 0.0
            && pos.y > 0.0
            && pos.z > 0.0
    }
}

impl Display for Volume {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        write!(
            fmt,
            "coords {} {} {} | scale {} {} {} | data length {} B",
            self.x,
            self.y,
            self.z,
            self.scale_x,
            self.scale_y,
            self.scale_z,
            self.data.len()
        )
    }
}
