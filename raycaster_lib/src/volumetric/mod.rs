//! All volume types are declared here.
//! They are re-exported as the module volumes.
//!
//! # What is a volume type?
//!
//! Volume type is a representation of volume in our library.
//! One example may be `LinearVolume` - a volume stored by slices in memory.
//! Another example is `StreamBlockVolume` - a volume stored by blocks and read from a file.
//!
//! # `Volume` trait
//!
//! All volume types interface with renderers using `Volume` trait.
//! This way, rendering can be generic over volume types.
//!
//! Another trait used for volumes is `Blocked`.
//! This trait allows renderer to access volume blocks (where applicable) and is used for parallel rendering.

mod block_volume;
mod empty_index;
mod float_block;
mod float_block_volume;
mod float_volume;
mod linear_volume;
mod vol_builder;
mod volume;

// Exports

pub use empty_index::EmptyIndex;
pub use vol_builder::DataSource;
pub use vol_builder::{BuildVolume, MemoryType, StorageShape, VolumeMetadata}; // todo move
pub use volume::{Blocked, Volume};

pub mod volumes {
    use super::*;

    pub use block_volume::{Block, BlockVolume};
    pub use float_block::FloatBlock;
    pub use float_block_volume::FloatBlockVolume;
    pub use float_volume::FloatVolume;
    pub use linear_volume::LinearVolume;
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::test_helpers::*;
    use nalgebra::{point, vector};
    use volumes::*;

    /// Float comparison for optional values
    fn compare_samples(s1: Option<f32>, s2: Option<f32>) -> bool {
        let x = match (s1, s2) {
            (None, None) => true,
            (None, Some(_)) => false,
            (Some(_), None) => false,
            (Some(v1), Some(v2)) => (v1 - v2).abs() < f32::EPSILON,
        };
        if !x {
            eprintln!("Comp. failed {s1:?} {s2:?}");
        }
        x
    }

    /// Expected: `Linearvolume` and `Blockvolume` match when sampled everywhere
    #[test]
    fn linear_block_matches() {
        let linear: FloatVolume = white_volume();
        let block: FloatBlockVolume = white_volume();

        let vol_size_l = linear.get_size();
        let vol_size_b = block.get_size();
        assert_eq!(vol_size_l, vol_size_b);

        for x in 0..vol_size_l.x {
            for y in 0..vol_size_l.y {
                for z in 0..vol_size_l.z {
                    let lin_data = linear.get_data(x, y, z);
                    let bl_data = block.get_data(x, y, z);

                    assert!(compare_samples(lin_data, bl_data));
                }
            }
        }
    }

    /// Same test as above, with bigger volume
    #[test]
    fn linear_block_matches_skull() {
        let linear: LinearVolume = skull_volume(None);
        let block: BlockVolume = skull_volume(Some(5));

        let vol_size_l = linear.get_size();
        let vol_size_b = block.get_size();
        assert_eq!(vol_size_l, vol_size_b);

        for x in 0..vol_size_l.x {
            for y in 0..vol_size_l.y {
                for z in 0..vol_size_l.z {
                    let lin_data = linear.get_data(x, y, z);
                    let bl_data = block.get_data(x, y, z);

                    assert!(compare_samples(lin_data, bl_data));
                }
            }
        }
    }

    /// Expected: Samples inside cell match
    #[test]
    fn sample_at_subsamples_match() {
        let linear: FloatVolume = skull_volume(None);
        let block: FloatBlockVolume = skull_volume(Some(5));

        let sampling_coord = point![0.0, 25.0, 114.0];
        let sampling_spots = [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9];

        for spot_offset in sampling_spots {
            let spot = sampling_coord + vector![spot_offset, spot_offset, spot_offset];
            let lin_sample = linear.sample_at(spot);
            let block_sample = block.sample_at(spot);
            let dif = (lin_sample - block_sample).abs();

            assert!(dif < f32::EPSILON);
        }
    }

    #[test]
    #[ignore]
    /// Test ignored as it a longer test
    /// Ignored tests can be run using `cargo test -- --ignored`
    fn linear_block_sample_at_matches() {
        let linear: FloatVolume = skull_volume(None);
        let block: FloatBlockVolume = skull_volume(Some(5));

        let vol_size_l = linear.get_size();
        let vol_size_b = block.get_size();
        assert_eq!(vol_size_l, vol_size_b);

        for x in 0..vol_size_l.x {
            for y in 0..vol_size_l.y {
                for z in 0..vol_size_l.z {
                    let pos = point![x as f32, y as f32, z as f32];
                    let lin_data = linear.sample_at(pos);
                    let bl_data = block.sample_at(pos);
                    let dif = (lin_data - bl_data).abs();

                    assert!(dif < f32::EPSILON);
                }
            }
        }
    }
}
