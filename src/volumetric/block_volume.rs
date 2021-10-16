use std::convert::TryInto;

use nalgebra::Vector3;

use super::{vol_builder::BuildVolume, VolumeBuilder};

pub struct Block {
    pub data: [u8; 8],
}

impl Block {
    pub fn new(data: [u8; 8]) -> Block {
        Block { data }
    }

    pub fn index(x: usize, y: usize, z: usize) -> usize {
        z + y * 2 + x * 4
    }
}

pub struct BlockVolume {
    levels: u8,
    binary_blocks: Vec<BlockLevel>,
    size: Vector3<usize>,
    data: Vec<Block>,
}

impl BlockVolume {
    pub fn base_blocklevel(builder: &VolumeBuilder) -> Vec<Block> {
        let mut blocks = vec![];

        for z in 0..builder.size.z {
            for y in 0..builder.size.y {
                for x in 0..builder.size.x {
                    let data = builder.get_surrounding_data(x, y, z);
                    let block = Block::new(data);
                    blocks.push(block);
                }
            }
        }

        blocks
    }

    fn build_binary_block(&self, level: u8) -> BlockLevel {
        match level {
            0 => panic!("Cant build level 0"),
            1 => BlockLevel::from_data(&self.data[..]),
            m => BlockLevel::from_block_level(&self.binary_blocks[(m - 1) as usize]),
        }
    }

    pub fn build_binary_blocks(&mut self) {
        for m in 1..self.levels {
            let b_level = self.build_binary_block(m);
            self.binary_blocks.push(b_level);
        }
    }
}

// return (side length, number of hierarchies)
fn lowest_volume_size(current: usize) -> (usize, usize) {
    let mut mul: usize = 0;
    loop {
        let number = 2_usize.pow(mul as u32) + 1;
        if number > current {
            return (number, mul);
        }
        mul += 1;
    }
}

impl BuildVolume for BlockVolume {
    fn build(builder: VolumeBuilder) -> BlockVolume {
        let longest_side = builder.size.max();
        let (padded_side, levels) = lowest_volume_size(longest_side);
        println!("padded {} m {}", padded_side, levels);

        let base_blocklevel = BlockLevel::base_blocklevel(&builder);
        let mut blocks = vec![base_blocklevel];

        for m in 1..levels {
            let level_m_blocks = vec![];

            let block_level = BlockLevel {
                blocks: level_m_blocks,
            };
            blocks.push(block_level);
        }

        BlockVolume {
            levels: levels as u8,
            blocks,
        }
    }
}
