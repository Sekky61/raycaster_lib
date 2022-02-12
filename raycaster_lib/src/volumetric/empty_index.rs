use nalgebra::{point, Point3, Vector3};

use super::Volume;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BlockType {
    Empty,
    NonEmpty,
}

impl BlockType {
    pub fn from_volume(volume: &impl Volume, base: Point3<usize>) -> BlockType {
        let mut block_iter = volume.get_block(BLOCK_SIDE, base);
        let is_empty = block_iter.all(|f| f == 0.0);
        match is_empty {
            true => BlockType::Empty,
            false => BlockType::NonEmpty,
        }
    }
}

const BLOCK_SIDE: usize = 2;

#[derive(Debug)]
pub struct EmptyIndex {
    size: Vector3<usize>,
    blocks: Vec<BlockType>,
}

impl EmptyIndex {
    // todo take transfer function into account
    pub fn from_volume(volume: &impl Volume) -> EmptyIndex {
        let index_size = (volume.get_size()) / BLOCK_SIDE;
        println!(
            "Generating index, size [{},{},{}]",
            index_size.x, index_size.y, index_size.z
        );

        let cell_count = index_size.iter().product();
        let mut blocks = Vec::with_capacity(cell_count);

        for x in 0..index_size.x {
            for y in 0..index_size.y {
                for z in 0..index_size.z {
                    let block_type = BlockType::from_volume(volume, point![x, y, z]);
                    blocks.push(block_type);
                }
            }
        }

        EmptyIndex {
            size: index_size,
            blocks,
        }
    }

    fn index_3d(&self, x: usize, y: usize, z: usize) -> usize {
        z + y * self.size.z + x * self.size.y * self.size.z
    }

    fn pos_to_index(&self, pos: Point3<f32>) -> usize {
        let x = pos.x as usize / BLOCK_SIDE;
        let y = pos.y as usize / BLOCK_SIDE;
        let z = pos.z as usize / BLOCK_SIDE;

        self.index_3d(x, y, z)
    }

    pub fn sample(&self, pos: Point3<f32>) -> BlockType {
        let index = self.pos_to_index(pos);
        self.blocks[index]
    }
}

#[cfg(test)]
mod test {

    use nalgebra::vector;

    use crate::volumetric::{
        vol_builder::{BuildVolume, DataSource},
        LinearVolume,
    };

    use super::*;

    fn volume_dims_empty(x: usize, y: usize, z: usize) -> LinearVolume {
        let (meta, vec) = crate::volumetric::empty_vol(vector![x, y, z]);
        BuildVolume::build(meta, DataSource::Vec(vec)).unwrap()
    }

    fn volume_dims_nonempty(x: usize, y: usize, z: usize) -> LinearVolume {
        let (meta, mut vec) = crate::volumetric::empty_vol(vector![x, y, z]);
        vec[2] = 17;
        BuildVolume::build(meta, DataSource::Vec(vec)).unwrap()
    }

    mod from_data {

        use super::*;

        #[test]
        fn empty() {
            let volume = volume_dims_empty(2, 2, 2);
            let empty_index = EmptyIndex::from_volume(&volume);

            assert_eq!(empty_index.blocks.len(), 1);
            assert_eq!(empty_index.blocks[0], BlockType::Empty);
            assert_eq!(empty_index.size, vector![1, 1, 1]);
        }

        #[test]
        fn empty_uneven() {
            let volume = volume_dims_empty(2, 3, 2);
            let empty_index = EmptyIndex::from_volume(&volume);

            assert_eq!(empty_index.blocks.len(), 2);
            assert_eq!(empty_index.blocks[0], BlockType::Empty);
            assert_eq!(empty_index.blocks[1], BlockType::Empty);
            assert_eq!(empty_index.size, vector![1, 2, 1]);
        }

        #[test]
        fn non_empty() {
            let volume = volume_dims_nonempty(2, 2, 2);
            let empty_index = EmptyIndex::from_volume(&volume);

            assert_eq!(empty_index.blocks.len(), 1);
            assert_eq!(empty_index.blocks[0], BlockType::NonEmpty);
            assert_eq!(empty_index.size, vector![1, 1, 1]);
        }
    }

    mod get_index {
        use super::*;

        #[test]
        fn base() {
            let volume = volume_dims_nonempty(4, 4, 4);
            let empty_index = EmptyIndex::from_volume(&volume);

            assert_eq!(empty_index.sample(point![0.0, 0.0, 0.0]), BlockType::Empty);
            assert_eq!(
                empty_index.sample(point![0.78, 0.55, 1.4]),
                BlockType::NonEmpty
            );
        }
    }
}
