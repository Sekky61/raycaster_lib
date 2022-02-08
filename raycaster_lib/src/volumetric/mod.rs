mod block_volume;
mod empty_index;
mod linear_volume;
pub mod parse;
mod stream_volume;
mod vol_builder;
mod volume;

pub use block_volume::BlockVolume;
pub use empty_index::{BlockType, EmptyIndex, EmptyIndexes};
pub use linear_volume::LinearVolume;
pub use stream_volume::StreamVolume;
pub use vol_builder::{BuildVolume, ParsedVolumeBuilder, VolumeBuilder};
pub use volume::Volume;

use nalgebra::{vector, Vector3};

pub fn white_vol() -> ParsedVolumeBuilder<u8> {
    ParsedVolumeBuilder {
        size: vector![2, 2, 2],
        border: 0,
        scale: vector![100.0, 100.0, 100.0], // shape of voxels
        data: Some(vec![0, 32, 64, 64 + 32, 128, 128 + 32, 128 + 64, 255]),
        mmap: None,
    }
}

pub fn empty_vol(dims: Vector3<usize>) -> ParsedVolumeBuilder<u8> {
    ParsedVolumeBuilder {
        size: dims,
        border: 0,
        scale: vector![100.0, 100.0, 100.0], // shape of voxels
        data: Some(vec![0; dims.x * dims.y * dims.z]),
        mmap: None,
    }
}

#[cfg(test)]
mod test {

    use super::*;

    use crate::volumetric::LinearVolume;
    use nalgebra::{point, vector};
    use vol_builder::BuildVolume;

    fn cube_volume<V>() -> V
    where
        V: Volume + BuildVolume<ParsedVolumeBuilder<u8>>,
    {
        let parsed_vb = white_vol();
        <V as BuildVolume<ParsedVolumeBuilder<u8>>>::build(parsed_vb)
    }

    fn skull_volume<V>() -> V
    where
        V: Volume + BuildVolume<ParsedVolumeBuilder<u8>>,
    {
        let vb = VolumeBuilder::from_file("volumes/Skull.vol").expect("skull error");
        let parsed_vb = match parse::skull_parser(vb) {
            Ok(res) => res,
            Err(err_msg) => panic!("{}", err_msg),
        };
        <V as BuildVolume<ParsedVolumeBuilder<u8>>>::build(parsed_vb)
    }

    #[test]
    fn linear_block_matches() {
        let linear: LinearVolume = cube_volume();
        let block: BlockVolume = cube_volume();

        let vol_size_l = linear.get_size();
        let vol_size_b = block.get_size();
        assert_eq!(vol_size_l, vol_size_b);

        for x in 0..vol_size_l.x {
            for y in 0..vol_size_l.y {
                for z in 0..vol_size_l.z {
                    let lin_data = linear.get_data(x, y, z);
                    let bl_data = block.get_data(x, y, z);
                    let dif = (lin_data - bl_data).abs();

                    assert!(dif < f32::EPSILON);
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
                    let dif = (lin_data - bl_data).abs();

                    assert!(dif < f32::EPSILON);
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
