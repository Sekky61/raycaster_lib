mod block_volume;
mod empty_index;
mod linear_volume;
mod stream_volume;
mod vol_builder;
mod volume;

pub use crate::color::RGBA;
pub use block_volume::BlockVolume;
pub use empty_index::{BlockType, EmptyIndex};
pub use linear_volume::LinearVolume;
pub use stream_volume::StreamVolume;
pub use vol_builder::DataSource;
pub use vol_builder::{BuildVolume, VolumeMetadata};
pub use volume::Volume;

pub type TF = fn(f32) -> RGBA;

#[cfg(test)]
mod test {

    use super::*;
    use crate::test_helpers::*;
    use crate::volumetric::LinearVolume;
    use nalgebra::{point, vector};

    fn compare_samples(s1: Option<f32>, s2: Option<f32>) -> bool {
        match (s1, s2) {
            (None, None) => true,
            (None, Some(_)) => false,
            (Some(_), None) => false,
            (Some(v1), Some(v2)) => (v1 - v2).abs() < f32::EPSILON,
        }
    }

    #[test]
    fn linear_block_matches() {
        let linear: LinearVolume = white_volume();
        let block: BlockVolume = white_volume();

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

    #[test] // #[ignore]
    fn linear_block_matches_skull() {
        let linear: LinearVolume = skull_volume();
        let block: BlockVolume = skull_volume();

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

    #[test]
    fn sample_at_subsamples_match() {
        let linear: LinearVolume = skull_volume();
        let block: BlockVolume = skull_volume();

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
    fn linear_block_sample_at_matches() {
        let linear: LinearVolume = skull_volume();
        let block: BlockVolume = skull_volume();

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
