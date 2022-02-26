use nalgebra::{vector, Vector3};

use crate::{
    color::RGBA,
    premade::parse::skull_parser,
    volumetric::{BuildVolume, DataSource, Volume, VolumeMetadata},
};

pub fn white_tf(sample: f32) -> RGBA {
    crate::color::mono(255.0, sample / 255.0)
}

pub fn white_vol_meta() -> VolumeMetadata<u8> {
    let data = vec![0, 32, 64, 64 + 32, 128, 128 + 32, 128 + 64, 255];
    let data_source = DataSource::Vec(data);
    VolumeMetadata {
        size: Some(vector![2, 2, 2]),
        scale: Some(vector![100.0, 100.0, 100.0]), // shape of voxels
        data_offset: Some(0),
        position: Some(vector![0.0, 0.0, 0.0]),
        data: Some(data_source),
        tf: Some(white_tf),
    }
}

pub fn empty_vol_meta(dims: Vector3<usize>) -> VolumeMetadata<u8> {
    let data = vec![0; dims.x * dims.y * dims.z];
    let data_source = DataSource::Vec(data);
    VolumeMetadata {
        size: Some(vector![2, 2, 2]),
        scale: Some(vector![100.0, 100.0, 100.0]), // shape of voxels
        data_offset: Some(0),
        position: Some(vector![0.0, 0.0, 0.0]),
        data: Some(data_source),
        tf: Some(white_tf),
    }
}

pub fn empty_volume<V>(dims: Vector3<usize>) -> V
where
    V: Volume + BuildVolume<u8>,
{
    let meta = empty_vol_meta(dims);
    BuildVolume::build(meta).unwrap()
}

pub fn white_volume<V>() -> V
where
    V: Volume + BuildVolume<u8>,
{
    let meta = white_vol_meta();
    BuildVolume::build(meta).unwrap()
}

pub fn skull_volume<V>() -> V
where
    V: Volume + BuildVolume<u8>,
{
    let ds = DataSource::from_file("volumes/Skull.vol").unwrap();
    let meta = skull_parser(ds).expect("skull error");
    BuildVolume::build(meta).unwrap()
}
