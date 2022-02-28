use std::path::Path;

use nalgebra::{vector, Vector3};
use nom::{
    bytes::complete::take,
    number::complete::{be_f32, be_u32, le_u16},
    sequence::tuple,
    IResult,
};

use crate::volumetric::{BuildVolume, DataSource, Volume, VolumeMetadata, TF};

use super::transfer_functions::{beetle_tf, skull_tf};

// Common pattern
pub fn from_file<P, T, M>(
    path: P,
    parser: fn(DataSource<u8>) -> Result<VolumeMetadata<M>, &'static str>,
    tf: TF,
) -> Result<T, &'static str>
where
    P: AsRef<Path>,
    T: BuildVolume<M> + Volume,
{
    let ds: DataSource<u8> = DataSource::from_file(path)?;
    let mut metadata = parser(ds)?;
    metadata.set_tf(tf);
    BuildVolume::build(metadata)
}

pub fn from_data_source<T, M>(
    ds: DataSource<u8>,
    parser: fn(&[u8]) -> Result<VolumeMetadata<M>, &'static str>,
    tf: TF,
) -> Result<T, &'static str>
where
    T: BuildVolume<M> + Volume,
{
    let slice = ds.get_slice().ok_or("Cannot get data")?;
    let mut metadata = parser(slice)?;
    metadata.set_tf(tf);
    BuildVolume::<M>::build(metadata)
}

// Little endian 2 byte values
// Values <0;4095>
pub fn beetle_parser(data_source: DataSource<u8>) -> Result<VolumeMetadata<u16>, &'static str> {
    // Scope to drop DataSource
    let size = {
        let mut beetle_header = tuple((le_u16, le_u16, le_u16));
        let slice = data_source.get_slice().ok_or("No data in data_source")?;
        let parse_res: IResult<_, _> = beetle_header(slice);

        let (_rest, size) = match parse_res {
            Ok(r) => r,
            Err(_) => return Err("Parse error"),
        };
        size
    };

    let new_data_src = data_source.into_transmute();

    let size = vector![size.0 as usize, size.1 as usize, size.2 as usize];

    let meta = VolumeMetadata {
        position: None,
        size: Some(size),
        scale: None,
        data: Some(new_data_src),
        data_offset: Some(6),
        tf: Some(beetle_tf),
    };

    Ok(meta)
}

pub struct ExtractedMeta {
    offset: usize,
    size: Vector3<usize>,
    scale: Vector3<f32>,
}

pub fn skull_parser(data_source: DataSource<u8>) -> Result<VolumeMetadata<u8>, &'static str> {
    let slice = data_source.get_slice().ok_or("No data in data_source")?;

    let parse_res = skull_inner(slice);

    let ExtractedMeta {
        offset,
        size,
        scale,
    } = match parse_res {
        Ok(r) => r.1,
        Err(_) => return Err("Parse error"),
    };

    Ok(VolumeMetadata {
        position: None,
        size: Some(size),
        scale: Some(scale),
        data_offset: Some(offset),
        data: Some(data_source),
        tf: Some(skull_tf),
    })
}

fn skull_inner(s: &[u8]) -> IResult<&[u8], ExtractedMeta> {
    let mut skull_header = tuple((
        tuple((be_u32, be_u32, be_u32)),
        take(4_u8),
        tuple((be_f32, be_f32, be_f32)),
    ));

    let (s, (size, _, scale)) = skull_header(s)?;

    let size = vector![size.0 as usize, size.1 as usize, size.2 as usize];
    let scale = vector![scale.0 * 0.999, scale.1 * 0.999, scale.2 * 0.999];

    // 4B * 7 = 28B
    let offset = 28;

    Ok((
        s,
        ExtractedMeta {
            offset,
            size,
            scale,
        },
    ))
}
