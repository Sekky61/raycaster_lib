pub mod vol_builder;
pub mod vol_reader;
pub mod volume;

mod block_volume;
mod empty_index;
mod linear_volume;

pub use block_volume::BlockVolume;
pub use empty_index::{BlockType, EmptyIndex, EmptyIndexes};
pub use linear_volume::LinearVolume;
pub use vol_builder::VolumeBuilder;
pub use volume::Volume;

#[cfg(test)]
mod test {

    use super::*;

    use crate::{vol_reader, volumetric::LinearVolume};
    use nalgebra::vector;
    use vol_builder::BuildVolume;

    fn cube_volume<V>() -> V
    where
        V: Volume + BuildVolume,
    {
        VolumeBuilder::white_vol().build()
    }

    fn skull_volume<V>() -> V
    where
        V: Volume + BuildVolume,
    {
        let builder = vol_reader::from_file("Skull.vol").expect("skull error");
        builder.build()
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

                    assert!(dif.iter().all(|&f| f < f32::EPSILON));
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

                    assert!(dif.iter().all(|&f| f < f32::EPSILON));
                }
            }
        }
    }

    #[test]
    fn sample_at_subsamples_match() {
        let linear: LinearVolume = skull_volume();
        let block: BlockVolume = skull_volume();

        let sampling_coord = vector![0.0, 25.0, 114.0];
        let sampling_spots = [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9];

        for spot_offset in sampling_spots {
            let spot = sampling_coord + vector![spot_offset, spot_offset, spot_offset];
            let lin_sample = linear.sample_at(spot);
            let block_sample = block.sample_at(spot);
            let dif = (lin_sample - block_sample).abs();

            assert!(dif.iter().all(|&f| f < f32::EPSILON));
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
                    let pos = vector![x as f32, y as f32, z as f32];
                    let lin_data = linear.sample_at(pos);
                    let bl_data = block.sample_at(pos);
                    let dif = (lin_data - bl_data).abs();

                    assert!(dif.iter().all(|&f| f < f32::EPSILON));
                }
            }
        }
    }
}
