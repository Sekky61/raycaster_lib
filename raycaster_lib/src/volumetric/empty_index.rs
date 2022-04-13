use nalgebra::{point, vector, Point3, Vector3};

use crate::{common::blockify, TF};

use super::{block::Block, Volume};

// generic argument S is side of block in voxels, not cells
#[derive(Debug)]
pub struct EmptyIndex<const S: usize> {
    size: Vector3<usize>,
    blocks: Vec<bool>, // True if empty
}

impl<const S: usize> EmptyIndex<S> {
    const DEM: f32 = 1.0 / (S as f32);

    /// Placeholder
    pub fn dummy() -> EmptyIndex<S> {
        EmptyIndex {
            size: vector![0, 0, 0],
            blocks: vec![],
        }
    }

    pub fn from_volume(volume: &impl Volume) -> EmptyIndex<S> {
        let vol_size = volume.get_size();
        let index_size = blockify(vol_size, S, 1);
        println!(
            "Generating index, vol [{},{},{}] size [{},{},{}]",
            vol_size.x, vol_size.y, vol_size.z, index_size.x, index_size.y, index_size.z
        );

        let cell_count = index_size.iter().product();
        let mut blocks = Vec::with_capacity(cell_count);

        let tf = volume.get_tf();

        for x in 0..index_size.x {
            for y in 0..index_size.y {
                for z in 0..index_size.z {
                    let block_type =
                        EmptyIndex::<S>::block_type_from_volume(volume, S * point![x, y, z], S, tf);
                    blocks.push(block_type);
                }
            }
        }

        EmptyIndex {
            size: index_size,
            blocks,
        }
    }

    /// returns true if block is empty
    pub fn block_type_from_volume(
        volume: &impl Volume,
        base: Point3<usize>,
        side: usize,
        tf: TF,
    ) -> bool {
        let block_iter = volume.get_block(side + 1, base); // side in voxels vs side in blocks

        // True if empty
        block_iter.map(tf).all(|f| f.w == 0.0)
    }

    fn index_3d(&self, x: usize, y: usize, z: usize) -> usize {
        z + y * self.size.z + x * self.size.y * self.size.z
    }

    fn pos_to_index(&self, pos: Point3<f32>) -> usize {
        let scaled_down = pos * EmptyIndex::<S>::DEM;
        let x = scaled_down.x as usize;
        let y = scaled_down.y as usize;
        let z = scaled_down.z as usize;

        self.index_3d(x, y, z)
    }

    pub fn is_empty(&self, pos: Point3<f32>) -> bool {
        let index = self.pos_to_index(pos);
        self.blocks[index]
    }

    pub fn is_empty_int(&self, pos: Point3<usize>) -> bool {
        let pos_conv = pos / S;
        let index = self.index_3d(pos_conv.x, pos_conv.y, pos_conv.z);
        self.blocks[index]
    }

    pub fn from_volume_without_tf(volume: &impl Volume, tf: TF) -> EmptyIndex<4> {
        let vol_size = volume.get_size();
        let index_size = blockify(vol_size, S, 1);

        #[cfg(debug_assertions)]
        println!(
            "Generating index, vol [{},{},{}] size [{},{},{}]",
            vol_size.x, vol_size.y, vol_size.z, index_size.x, index_size.y, index_size.z
        );

        let cell_count = index_size.iter().product();
        let mut blocks = Vec::with_capacity(cell_count);

        for x in 0..index_size.x {
            for y in 0..index_size.y {
                for z in 0..index_size.z {
                    let block_type =
                        EmptyIndex::<S>::block_type_from_volume(volume, S * point![x, y, z], S, tf);
                    blocks.push(block_type);
                }
            }
        }

        EmptyIndex {
            size: index_size,
            blocks,
        }
    }
}

#[cfg(test)]
mod test {

    const EMPTY: bool = true;
    const NONEMPTY: bool = false;

    use super::*;
    use crate::{test_helpers::*, volumetric::linear_volume::LinearVolume};
    use nalgebra::vector;

    use crate::volumetric::{
        vol_builder::{BuildVolume, DataSource},
        RGBA,
    };

    fn dark_tf(_sample: f32) -> RGBA {
        crate::color::zero()
    }

    fn volume_dims_nonempty(dims: Vector3<usize>, non_empty_indexes: &[usize]) -> LinearVolume {
        let mut vol = empty_vol_meta(dims);
        if let Some(ref mut data) = vol.data {
            match data {
                DataSource::Vec(ref mut v) => {
                    for &i in non_empty_indexes {
                        v[i] = 1;
                    }
                }
                _ => panic!("Data source not a vector"),
            }
        } else {
            panic!("test error - no data from empty_vol");
        }

        BuildVolume::build(vol).unwrap()
    }

    mod from_data {

        use super::*;

        #[test]
        fn empty() {
            let volume: LinearVolume = empty_volume(vector![2, 2, 2]);
            let empty_index = EmptyIndex::<2>::from_volume(&volume);

            assert_eq!(volume.get_size().iter().product::<usize>(), 8);
            assert_eq!(empty_index.blocks.len(), 1);
            assert_eq!(empty_index.blocks[0], EMPTY);
            assert_eq!(empty_index.size, vector![1, 1, 1]);
        }

        #[test]
        fn empty_bigger() {
            let volume: LinearVolume = empty_volume(vector![24, 24, 10]);
            let empty_index = EmptyIndex::<3>::from_volume(&volume);

            assert_eq!(empty_index.blocks.len(), 12 * 12 * 5);
            assert_eq!(empty_index.blocks[0], EMPTY);
            assert_eq!(empty_index.size, vector![12, 12, 5]);
        }

        #[test]
        fn non_empty() {
            let volume: LinearVolume = volume_dims_nonempty(vector![2, 2, 2], &[2]);
            let empty_index = EmptyIndex::<2>::from_volume(&volume);

            assert_eq!(empty_index.blocks.len(), 1);
            assert_eq!(empty_index.blocks[0], NONEMPTY);
            assert_eq!(empty_index.size, vector![1, 1, 1]);
        }

        #[test]
        fn empty_side3() {
            // cell size 3 --> 4 voxels
            let volume: LinearVolume = empty_volume(vector![10, 5, 19]);
            let empty_index = EmptyIndex::<4>::from_volume(&volume);

            assert_eq!(empty_index.blocks.len(), 3 * 2 * 6);
            assert_eq!(empty_index.blocks[4], EMPTY);
            assert_eq!(empty_index.size, vector![3, 2, 6]);
        }

        #[test]
        fn empty_side6() {
            // cell size 6 --> 7 voxels
            let volume: LinearVolume = empty_volume(vector![23, 15, 8]);
            let empty_index = EmptyIndex::<7>::from_volume(&volume);

            assert_eq!(empty_index.blocks.len(), 4 * 3 * 2);
            assert_eq!(empty_index.blocks[2], EMPTY);
            assert_eq!(empty_index.size, vector![4, 3, 2]);
        }

        // Index takes into account resulting opacity, not values of samples
        #[test]
        fn empty_dark_tf() {
            let mut meta = empty_vol_meta(vector![7, 7, 7]);
            meta.set_tf(dark_tf);

            if let Some(ref mut data) = meta.data {
                match data {
                    DataSource::Vec(ref mut v) => v[2] = 20,
                    _ => panic!("Data source not a vector"),
                }
            } else {
                panic!("test error - no data from empty_vol");
            }

            let volume: LinearVolume = BuildVolume::build(meta).unwrap();

            let empty_index = EmptyIndex::<2>::from_volume(&volume);

            assert!(empty_index.blocks.iter().all(|&b| b == EMPTY));
        }
    }

    mod get_index {
        use super::*;

        #[test]
        fn base() {
            let volume = volume_dims_nonempty(vector![5, 5, 5], &[1]);
            let empty_index = EmptyIndex::<2>::from_volume(&volume);

            assert_eq!(empty_index.blocks.len(), 4 * 4 * 4);
            assert_eq!(empty_index.is_empty(point![0.0, 0.0, 0.0]), NONEMPTY);
            assert_eq!(empty_index.is_empty(point![1.7, 1.5, 1.4]), NONEMPTY);
            assert_eq!(empty_index.is_empty(point![2.1, 1.55, 1.4]), EMPTY);
            assert_eq!(empty_index.is_empty(point![0.1, 2.2, 1.4]), EMPTY);
            assert_eq!(empty_index.is_empty(point![0.1, 1.5, 2.4]), EMPTY);
        }

        #[test]
        fn border_nonempty() {
            let volume = volume_dims_nonempty(vector![5, 5, 5], &[2]);
            let empty_index = EmptyIndex::<2>::from_volume(&volume);

            println!("blocks: {:?}", &empty_index.blocks[..]);

            assert_eq!(empty_index.blocks.len(), 4 * 4 * 4);
            assert_eq!(empty_index.is_empty(point![0.0, 0.0, 0.0]), NONEMPTY);
            assert_eq!(empty_index.is_empty(point![1.7, 1.5, 1.4]), NONEMPTY);
            assert_eq!(empty_index.is_empty(point![0.1, 0.5, 2.4]), NONEMPTY);
            assert_eq!(empty_index.is_empty(point![2.1, 1.55, 1.4]), EMPTY);
        }
    }
}
