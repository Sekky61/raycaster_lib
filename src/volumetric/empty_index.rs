use nalgebra::{vector, Vector3};

use super::Volume;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BlockType {
    Empty,
    NonEmpty,
}

#[derive(Default)]
pub struct EmptyIndexes {
    indexes: Vec<EmptyIndex>,
}

impl EmptyIndexes {
    pub fn get_index_size(m: usize) -> usize {
        2u32.pow(m as u32) as usize
    }

    pub fn get_block_coords(level: usize, pos: Vector3<f32>) -> Vector3<usize> {
        let low_pos = pos.map(|f| f.floor() as usize);

        let index_block_size = EmptyIndexes::get_index_size(level);

        low_pos.map(|p| p / index_block_size)
    }

    pub fn from_volume(volume: &impl Volume) -> EmptyIndexes {
        let mut indexes = vec![EmptyIndex::from_data(volume)];
        while let Some(ind) = EmptyIndex::from_empty_index(indexes.last().unwrap()) {
            indexes.push(ind);
        }

        EmptyIndexes { indexes }
    }

    pub fn get_index_at(&self, level: usize, pos: Vector3<f32>) -> BlockType {
        assert!(level < self.indexes.len());

        let block_pos = EmptyIndexes::get_block_coords(level, pos);

        let index_3d = self.get_3d_index_level(level, &block_pos);

        self.indexes[level].blocks[index_3d]
    }

    pub fn get_3d_index_level(&self, level: usize, block_pos: &Vector3<usize>) -> usize {
        let index_size = self.indexes[level].size;
        block_pos.z + block_pos.y * index_size.z + block_pos.x * index_size.y * index_size.z
    }

    pub fn len(&self) -> usize {
        self.indexes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub struct EmptyIndex {
    size: Vector3<usize>,
    blocks: Vec<BlockType>,
}

impl EmptyIndex {
    fn from_empty_index(base: &EmptyIndex) -> Option<EmptyIndex> {
        let base_dims = base.size;
        if base_dims.iter().any(|&x| x == 1) {
            // Bigger index does not make sense (I hope)
            return None;
        }
        let index_size = vector![
            (base_dims.x + 1) / 2,
            (base_dims.y + 1) / 2,
            (base_dims.z + 1) / 2
        ];
        let cell_count = index_size.iter().product();
        let mut blocks = Vec::with_capacity(cell_count);

        for x in 0..index_size.x {
            for y in 0..index_size.y {
                for z in 0..index_size.z {
                    let block_type = EmptyIndex::join_blocks(base, x, y, z);
                    blocks.push(block_type);
                }
            }
        }
        Some(EmptyIndex {
            size: index_size,
            blocks,
        })
    }

    fn from_data(volume: &impl Volume) -> EmptyIndex {
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

    fn volume_dims_empty(x: usize, y: usize, z: usize) -> LinearVolume {
        VolumeBuilder::new()
            .set_size(vector![x, y, z])
            .set_data(vec![0.0; x * y * z])
            .build()
    }

    fn volume_dims_nonempty(x: usize, y: usize, z: usize) -> LinearVolume {
        let mut data = vec![0.0; x * y * z];
        data[2] = 17.0;
        VolumeBuilder::new()
            .set_size(vector![x, y, z])
            .set_data(data)
            .build()
    }

    mod from_data {

        use super::*;

        #[test]
        fn empty() {
            let volume = volume_dims_empty(2, 2, 2);
            let empty_index = EmptyIndex::from_data(&volume);

            assert_eq!(empty_index.blocks.len(), 1);
            assert_eq!(empty_index.blocks[0], BlockType::Empty);
            assert_eq!(empty_index.size, vector![1, 1, 1]);
        }

        #[test]
        fn empty_uneven() {
            let volume = volume_dims_empty(2, 3, 2);
            let empty_index = EmptyIndex::from_data(&volume);

            assert_eq!(empty_index.blocks.len(), 2);
            assert_eq!(empty_index.blocks[0], BlockType::Empty);
            assert_eq!(empty_index.blocks[1], BlockType::Empty);
            assert_eq!(empty_index.size, vector![1, 2, 1]);
        }

        #[test]
        fn non_empty() {
            let volume = volume_dims_nonempty(2, 2, 2);
            let empty_index = EmptyIndex::from_data(&volume);

            assert_eq!(empty_index.blocks.len(), 1);
            assert_eq!(empty_index.blocks[0], BlockType::NonEmpty);
            assert_eq!(empty_index.size, vector![1, 1, 1]);
        }
    }

    mod from_empty_index {

        use super::*;

        mod level_1 {
            use super::*;

            #[test]
            fn cube() {
                let volume = volume_dims_empty(3, 3, 3);
                let empty_index = EmptyIndex::from_data(&volume);
                let level_1 = EmptyIndex::from_empty_index(&empty_index).unwrap();

                assert_eq!(level_1.blocks.len(), 1);
                assert_eq!(level_1.blocks[0], BlockType::Empty);
                assert_eq!(level_1.size, vector![1, 1, 1]);
            }

            #[test]
            fn too_small() {
                let volume = volume_dims_empty(2, 3, 2);
                let empty_index = EmptyIndex::from_data(&volume);
                let level_1 = EmptyIndex::from_empty_index(&empty_index);

                assert!(level_1.is_none());
            }

            #[test]
            fn level_1_enough_4x4x4() {
                let volume = volume_dims_empty(4, 4, 4);
                let empty_index = EmptyIndex::from_data(&volume);
                let level_1 = EmptyIndex::from_empty_index(&empty_index).unwrap();

                assert_eq!(level_1.blocks.len(), 2 * 2 * 2);
                assert_eq!(level_1.blocks[0], BlockType::Empty);
                assert_eq!(level_1.size, vector![2, 2, 2]);
            }

            #[test]
            fn uneven() {
                let volume = volume_dims_empty(4, 3, 3);
                let empty_index = EmptyIndex::from_data(&volume);
                let level_1 = EmptyIndex::from_empty_index(&empty_index).unwrap();

                assert_eq!(level_1.size, vector![2, 1, 1]);
                assert_eq!(level_1.blocks.len(), 2);
                level_1
                    .blocks
                    .iter()
                    .for_each(|&bl| assert_eq!(bl, BlockType::Empty));
            }
        }

        mod level_2 {
            use super::*;

            #[test]
            fn cube() {
                let volume = volume_dims_empty(5, 5, 5);
                let empty_index = EmptyIndex::from_data(&volume);
                let level_1 = EmptyIndex::from_empty_index(&empty_index).unwrap();
                let level_2 = EmptyIndex::from_empty_index(&level_1).unwrap();

                assert_eq!(level_2.size, vector![1, 1, 1]);
                assert_eq!(level_2.blocks.len(), 1);
                assert_eq!(level_2.blocks[0], BlockType::Empty);
            }

            #[test]
            fn uneven() {
                let volume = volume_dims_empty(9, 5, 5);
                let empty_index = EmptyIndex::from_data(&volume);
                let level_1 = EmptyIndex::from_empty_index(&empty_index).unwrap();
                let level_2 = EmptyIndex::from_empty_index(&level_1).unwrap();

                assert_eq!(level_2.size, vector![2, 1, 1]);
                assert_eq!(level_2.blocks.len(), 2);
                level_2
                    .blocks
                    .iter()
                    .for_each(|&bl| assert_eq!(bl, BlockType::Empty));
            }
        }
    }

    mod get_index {
        use super::*;

        #[test]
        fn base() {
            let volume = volume_dims_nonempty(4, 4, 4);
            let empty_index = EmptyIndexes::from_volume(&volume);

            assert_eq!(
                empty_index.get_index_at(0, vector![0.0, 0.0, 0.0]),
                BlockType::Empty
            );
            assert_eq!(
                empty_index.get_index_at(0, vector![0.78, 0.55, 1.4]),
                BlockType::NonEmpty
            );
        }

        #[test]
        fn level_1() {
            let volume = volume_dims_empty(4, 4, 4);
            let empty_index = EmptyIndexes::from_volume(&volume);

            assert_eq!(
                empty_index.get_index_at(1, vector![0.6, 0.5, 0.4]),
                BlockType::Empty
            );
            assert_eq!(
                empty_index.get_index_at(1, vector![2.1, 2.1, 1.2]),
                BlockType::Empty
            );
        }

        #[test]
        fn level_2() {
            let volume = volume_dims_empty(5, 5, 5);
            let empty_index = EmptyIndexes::from_volume(&volume);

            assert_eq!(
                empty_index.get_index_at(2, vector![0.6, 0.5, 0.4]),
                BlockType::Empty
            );
            assert_eq!(
                empty_index.get_index_at(2, vector![1.1, 3.1, 3.2]),
                BlockType::Empty
            );
        }
    }
}
