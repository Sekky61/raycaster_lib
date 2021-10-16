use nalgebra::{vector, Vector3};

use super::Volume;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BlockType {
    Empty,
    NonEmpty,
}

pub struct EmptyIndex {
    size: Vector3<usize>,
    blocks: Vec<BlockType>,
}

impl EmptyIndex {
    pub fn from_finer_empty_index(base: &EmptyIndex) -> EmptyIndex {
        let dims = base.size;
        let cell_count = dims.iter().fold(1, |acc, x| acc * (x - 1));
        let mut blocks = Vec::with_capacity(cell_count);

        for x in 0..(dims.x - 1) {
            for y in 0..(dims.y - 1) {
                for z in 0..(dims.z - 1) {
                    let block_type = EmptyIndex::join_blocks(base, x, y, z);
                    blocks.push(block_type);
                }
            }
        }

        EmptyIndex { size: dims, blocks }
    }

    pub fn from_volume(volume: &impl Volume) -> EmptyIndex {
        // number of cells
        // example: 8 cells in 3x3x3 voxels
        let vol_dims = volume.get_size();
        let index_size = vector![vol_dims.x - 1, vol_dims.y - 1, vol_dims.z - 1];
        let cell_count = index_size.iter().product();
        let mut blocks = Vec::with_capacity(cell_count);

        for x in 0..index_size.x {
            for y in 0..index_size.y {
                for z in 0..index_size.z {
                    let block_type = EmptyIndex::data_block_type(volume, x, y, z);
                    blocks.push(block_type);
                }
            }
        }

        EmptyIndex {
            size: index_size,
            blocks,
        }
    }

    fn data_block_type(volume: &impl Volume, x: usize, y: usize, z: usize) -> BlockType {
        let samples = [
            volume.get_data(x, y, z),
            volume.get_data(x, y, z + 1),
            volume.get_data(x, y + 1, z),
            volume.get_data(x, y + 1, z + 1),
            volume.get_data(x + 1, y, z),
            volume.get_data(x + 1, y, z + 1),
            volume.get_data(x + 1, y + 1, z),
            volume.get_data(x + 1, y + 1, z + 1),
        ];

        let any_nonzero = samples.iter().any(|&val| val != 0.0);
        match any_nonzero {
            true => BlockType::NonEmpty,
            false => BlockType::Empty,
        }
    }

    fn index_3d(&self, x: usize, y: usize, z: usize) -> usize {
        z + y * self.size.z + x * self.size.y * self.size.z
    }

    fn get_block(&self, x: usize, y: usize, z: usize) -> BlockType {
        self.blocks
            .get(self.index_3d(x, y, z))
            .cloned()
            .unwrap_or(BlockType::Empty)
    }

    fn join_blocks(index: &EmptyIndex, x: usize, y: usize, z: usize) -> BlockType {
        let samples = [
            index.get_block(x, y, z),
            index.get_block(x, y, z + 1),
            index.get_block(x, y + 1, z),
            index.get_block(x, y + 1, z + 1),
            index.get_block(x + 1, y, z),
            index.get_block(x + 1, y, z + 1),
            index.get_block(x + 1, y + 1, z),
            index.get_block(x + 1, y + 1, z + 1),
        ];

        let any_nonempty = samples.iter().any(|&val| val == BlockType::NonEmpty);
        match any_nonempty {
            true => BlockType::NonEmpty,
            false => BlockType::Empty,
        }
    }
}

#[cfg(test)]
mod test {

    use crate::volumetric::{LinearVolume, VolumeBuilder};
    use nalgebra::vector;

    use super::*;

    fn volume_2x2_empty() -> LinearVolume {
        VolumeBuilder::new()
            .set_size(vector![2, 2, 2])
            .set_data(vec![0.0; 8])
            .build()
    }

    fn volume_2x2_nonempty() -> LinearVolume {
        VolumeBuilder::new()
            .set_size(vector![2, 2, 2])
            .set_data(vec![0.0, 0.0, 0.0, 4.0, 0.0, 0.0, 0.0, 0.0])
            .build()
    }

    mod from_volume {

        use super::*;

        #[test]
        fn empty() {
            let volume = volume_2x2_empty();
            let empty_index = EmptyIndex::from_volume(&volume);

            assert_eq!(empty_index.blocks.len(), 1);
            assert_eq!(empty_index.blocks[0], BlockType::Empty);
            assert_eq!(empty_index.size, vector![1, 1, 1]);
        }
    }
}
