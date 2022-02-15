use nalgebra::{point, vector, Point3, Vector3};

use super::Volume;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BlockType {
    Empty,
    NonEmpty,
}

impl BlockType {
    pub fn from_volume(volume: &impl Volume, base: Point3<usize>, side: usize) -> BlockType {
        let block_iter = volume.get_block(side + 1, base); // side in voxels vs side in blocks
        let tf = volume.get_tf();
        let is_empty = block_iter.map(tf).all(|f| f.w == 0.0);
        match is_empty {
            true => BlockType::Empty,
            false => BlockType::NonEmpty,
        }
    }
}

#[derive(Debug)]
pub struct EmptyIndex<const S: usize> {
    size: Vector3<usize>,
    blocks: Vec<BlockType>,
}

impl<const S: usize> EmptyIndex<S> {
    pub fn from_volume(volume: &impl Volume) -> EmptyIndex<S> {
        let vol_size = volume.get_size();
        let index_size = (vol_size + vector![S - 2, S - 2, S - 2]) / S;
        println!(
            "Generating index, vol [{},{},{}] size [{},{},{}]",
            vol_size.x, vol_size.y, vol_size.z, index_size.x, index_size.y, index_size.z
        );

        let cell_count = index_size.iter().product();
        let mut blocks = Vec::with_capacity(cell_count);

        for x in 0..index_size.x {
            for y in 0..index_size.y {
                for z in 0..index_size.z {
                    let block_type = BlockType::from_volume(volume, S * point![x, y, z], S);
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
        let x = pos.x as usize / S;
        let y = pos.y as usize / S;
        let z = pos.z as usize / S;

        self.index_3d(x, y, z)
    }

    pub fn sample(&self, pos: Point3<f32>) -> BlockType {
        let index = self.pos_to_index(pos);
        self.blocks[index]
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use nalgebra::vector;

    use crate::volumetric::{
        vol_builder::{BuildVolume, DataSource},
        LinearVolume, RGBA,
    };

    fn volume_dims_empty(x: usize, y: usize, z: usize) -> LinearVolume {
        let (meta, vec) = crate::volumetric::empty_vol(vector![x, y, z]);
        BuildVolume::build(meta, DataSource::Vec(vec), crate::volumetric::white_tf).unwrap()
    }

    fn dark_tf(_sample: f32) -> RGBA {
        crate::color::zero()
    }

    fn volume_dims_nonempty(
        x: usize,
        y: usize,
        z: usize,
        non_empty_indexes: &[usize],
    ) -> LinearVolume {
        let (meta, mut vec) = crate::volumetric::empty_vol(vector![x, y, z]);
        for &i in non_empty_indexes {
            vec[i] = 1;
        }
        BuildVolume::build(meta, DataSource::Vec(vec), crate::volumetric::white_tf).unwrap()
    }

    mod from_data {

        use super::*;

        #[test]
        fn empty() {
            let volume = volume_dims_empty(2, 2, 2);
            let empty_index = EmptyIndex::<2>::from_volume(&volume);

            assert_eq!(volume.get_size().iter().product::<usize>(), 8);
            assert_eq!(empty_index.blocks.len(), 1);
            assert_eq!(empty_index.blocks[0], BlockType::Empty);
            assert_eq!(empty_index.size, vector![1, 1, 1]);
        }

        #[test]
        fn empty_bigger() {
            let volume = volume_dims_empty(24, 24, 10);
            let empty_index = EmptyIndex::<2>::from_volume(&volume);

            assert_eq!(empty_index.blocks.len(), 12 * 12 * 5);
            assert_eq!(empty_index.blocks[0], BlockType::Empty);
            assert_eq!(empty_index.size, vector![12, 12, 5]);
        }

        #[test]
        fn non_empty() {
            let volume = volume_dims_nonempty(2, 2, 2, &[2]);
            let empty_index = EmptyIndex::<2>::from_volume(&volume);

            assert_eq!(empty_index.blocks.len(), 1);
            assert_eq!(empty_index.blocks[0], BlockType::NonEmpty);
            assert_eq!(empty_index.size, vector![1, 1, 1]);
        }

        #[test]
        fn empty_side3() {
            let volume = volume_dims_empty(10, 5, 18);
            let empty_index = EmptyIndex::<3>::from_volume(&volume);

            assert_eq!(empty_index.blocks.len(), 3 * 2 * 6);
            assert_eq!(empty_index.blocks[4], BlockType::Empty);
            assert_eq!(empty_index.size, vector![3, 2, 6]);
        }

        #[test]
        fn empty_side6() {
            let volume = volume_dims_empty(23, 15, 8);
            let empty_index = EmptyIndex::<6>::from_volume(&volume);

            assert_eq!(empty_index.blocks.len(), 4 * 3 * 2);
            assert_eq!(empty_index.blocks[2], BlockType::Empty);
            assert_eq!(empty_index.size, vector![4, 3, 2]);
        }

        // Index takes into account resulting opacity, not values of samples
        #[test]
        fn empty_dark_tf() {
            let (meta, mut vec) = crate::volumetric::empty_vol(vector![7, 7, 7]);
            vec[2] = 20;
            let volume: LinearVolume =
                BuildVolume::build(meta, DataSource::Vec(vec), dark_tf).unwrap();

            let empty_index = EmptyIndex::<2>::from_volume(&volume);

            assert!(empty_index.blocks.iter().all(|&b| b == BlockType::Empty));
        }
    }

    mod get_index {
        use super::*;

        #[test]
        fn base() {
            let volume = volume_dims_nonempty(5, 5, 5, &[1]);
            let empty_index = EmptyIndex::<2>::from_volume(&volume);

            assert_eq!(empty_index.blocks.len(), 8);
            assert_eq!(
                empty_index.sample(point![0.0, 0.0, 0.0]),
                BlockType::NonEmpty
            );
            assert_eq!(
                empty_index.sample(point![1.7, 1.5, 1.4]),
                BlockType::NonEmpty
            );
            assert_eq!(empty_index.sample(point![2.1, 1.55, 1.4]), BlockType::Empty);
            assert_eq!(empty_index.sample(point![0.1, 2.2, 1.4]), BlockType::Empty);
            assert_eq!(empty_index.sample(point![0.1, 1.5, 2.4]), BlockType::Empty);
        }

        #[test]
        fn border_nonempty() {
            let volume = volume_dims_nonempty(5, 5, 5, &[2]);
            let empty_index = EmptyIndex::<2>::from_volume(&volume);

            println!("blocks: {:?}", &empty_index.blocks[..]);

            assert_eq!(empty_index.blocks.len(), 8);
            assert_eq!(
                empty_index.sample(point![0.0, 0.0, 0.0]),
                BlockType::NonEmpty
            );
            assert_eq!(
                empty_index.sample(point![1.7, 1.5, 1.4]),
                BlockType::NonEmpty
            );
            assert_eq!(
                empty_index.sample(point![0.1, 0.5, 2.4]),
                BlockType::NonEmpty
            );
            assert_eq!(empty_index.sample(point![2.1, 1.55, 1.4]), BlockType::Empty);
        }
    }
}
