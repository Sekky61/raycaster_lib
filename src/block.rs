pub const BLOCK_SIZE: usize = 16;

pub enum BlockType {
    Empty,
    NonEmpty,
}

pub struct Block {
    pub data: Vec<u8>,
}

impl Block {
    pub fn new() -> Block {
        Block {
            data: vec![0; BLOCK_SIZE * BLOCK_SIZE],
        }
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

    pub fn get_block(&self, x: usize, y: usize, z: usize) -> Option<&Block> {
        let block_x = (x as f32 / BLOCK_SIZE as f32).ceil() as usize;
        let block_y = (y as f32 / BLOCK_SIZE as f32).ceil() as usize;
        let block_z = (z as f32 / BLOCK_SIZE as f32).ceil() as usize;

        self.blocks.get(
            block_x + block_y * self.blocks_dim.0 + block_z * self.blocks_dim.0 * self.blocks_dim.1,
        )
    }
}
