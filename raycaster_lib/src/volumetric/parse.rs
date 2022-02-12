use super::vol_builder::VolumeMetadata;
use nalgebra::{vector, Vector3};
use nom::{
    bytes::complete::take,
    number::complete::{be_f32, be_u32, le_u16},
    sequence::tuple,
    IResult,
};

pub fn beetle_parser(slice: &[u8]) -> Result<VolumeMetadata, &'static str> {
    let mut beetle_header = tuple((le_u16, le_u16, le_u16));
    let parse_res: IResult<_, _> = beetle_header(slice);

    let (_rest, size) = match parse_res {
        Ok(r) => r,
        Err(_) => return Err("Parse error"),
    };

    // offset 2B * 3 = 6B

    let size = vector![size.0 as usize, size.1 as usize, size.2 as usize];

    let meta = VolumeMetadata {
        size,
        border: 0,
        scale: vector![1.0 * 0.99, 1.0 * 0.99, 1.0 * 0.99],
        data_offset: 6,
    };

    Ok(meta)
}

pub struct ExtractedMeta {
    offset: usize,
    size: Vector3<usize>,
    scale: Vector3<f32>,
}

pub fn skull_parser(slice: &[u8]) -> Result<VolumeMetadata, &'static str> {
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
        size,
        border: 0,
        scale,
        data_offset: offset,
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
