//! Module with helper functions
//! Saves repetition in unit tests

use nalgebra::{point, vector, Vector3};

use crate::{
    color::RGBA,
    premade::parse::skull_parser,
    volumetric::{BuildVolume, DataSource, StorageShape, Volume, VolumeMetadata},
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
        position: Some(point![0.0, 0.0, 0.0]),
        data: Some(data_source),
        tf: Some(white_tf),
        data_shape: Some(StorageShape::Linear),
        memory_type: None,
        desired_data_shape: None,
    }
}

pub fn empty_vol_meta(size: Vector3<usize>) -> VolumeMetadata<u8> {
    let data = vec![0; size.x * size.y * size.z];
    let data_source = DataSource::Vec(data);
    VolumeMetadata {
        size: Some(size),
        scale: Some(vector![100.0, 100.0, 100.0]), // shape of voxels
        position: Some(point![0.0, 0.0, 0.0]),
        data: Some(data_source),
        tf: Some(white_tf),
        data_shape: Some(StorageShape::Linear),
        memory_type: None,
        desired_data_shape: None,
    }
}

pub fn empty_volume<V>(size: Vector3<usize>) -> V
where
    V: Volume + BuildVolume<u8>,
{
    let meta = empty_vol_meta(size);
    BuildVolume::build(meta).unwrap()
}

pub fn white_volume<V>() -> V
where
    V: Volume + BuildVolume<u8>,
{
    let meta = white_vol_meta();
    BuildVolume::build(meta).unwrap()
}

/// Parses and returns volume from file `volumes/Skull.vol` (relative to workspaces)
/// Optional parameter `block_side` allows creation of block volumes
pub fn skull_volume<V>(block_side: Option<u8>) -> V
where
    V: Volume + BuildVolume<u8>,
{
    let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")); // should be library root (!not workspace dir!)
    path.push("../volumes/Skull.vol");
    println!("{:?}", path);
    let ds = DataSource::from_file(path).unwrap();
    let mut meta = skull_parser(ds).expect("skull error");
    match block_side {
        Some(side) => meta.desired_data_shape = Some(StorageShape::Z(side)),
        None => (),
    }
    BuildVolume::build(meta).unwrap()
}
