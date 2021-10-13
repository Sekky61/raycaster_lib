pub const BLOCK_SIZE: usize = 8;

use std::collections::HashMap;

// data: map of slices of block
pub struct BlockBuilder {
    data: HashMap<u8, Vec<u8>>,
}

impl BlockBuilder {
    pub fn new() -> BlockBuilder {
        BlockBuilder {
            data: HashMap::new(),
        }
    }
}

pub enum BlockType {
    Empty,
    NonEmpty,
}

pub struct Block {
    block_type: BlockType,
    pub data: Vec<u8>,
}

impl Block {
    pub fn new() -> Block {
        Block {
            block_type: BlockType::NonEmpty,
            data: vec![0; BLOCK_SIZE * BLOCK_SIZE * BLOCK_SIZE],
        }
    }

    pub fn from_data(data: Vec<u8>) -> Block {
        let non_zero_byte = data.iter().any(|&p| p != 0);
        let block_type = match non_zero_byte {
            true => BlockType::NonEmpty,
            false => BlockType::Empty,
        };

        if data.len() != BLOCK_SIZE * BLOCK_SIZE * BLOCK_SIZE {
            panic!("Data size not precisely matching block size");
        }

        Block { block_type, data }
    }

    pub fn get_data(&self, x: usize, y: usize, z: usize) -> u8 {
        self.data[x + y * BLOCK_SIZE + z * BLOCK_SIZE * BLOCK_SIZE]
    }
}

pub struct Blocks {
    blocks_dim: (usize, usize, usize),
    blocks: Vec<Block>,
}

impl Blocks {
    pub fn new() -> Blocks {
        Blocks {
            blocks_dim: (0, 0, 0),
            blocks: vec![],
        }
    }

    // get block coresponding to a voxel coordinates
    pub fn get_block(&self, x: usize, y: usize, z: usize) -> Option<&Block> {
        let block_x = (x as f32 / BLOCK_SIZE as f32).ceil() as usize;
        let block_y = (y as f32 / BLOCK_SIZE as f32).ceil() as usize;
        let block_z = (z as f32 / BLOCK_SIZE as f32).ceil() as usize;

        self.blocks.get(
            block_x + block_y * self.blocks_dim.0 + block_z * self.blocks_dim.0 * self.blocks_dim.1,
        )
    }
}
