use super::{
    vol_builder::{DataSource, TypedMmap},
    ParsedVolumeBuilder, VolumeBuilder,
};
use nalgebra::{vector, Vector3};
use nom::{
    bytes::complete::take,
    number::complete::{be_f32, be_u32, le_u16},
    sequence::tuple,
    IResult,
};

pub fn beetle_parser(vb: VolumeBuilder) -> Result<ParsedVolumeBuilder<u16>, &'static str> {
    let slice = if let Some(ref mmap) = vb.mmap {
        &mmap[..]
    } else if let Some(ref vec) = vb.data {
        &vec[..]
    } else {
        return Err("No data in VolumeBuilder");
    };

    let mut beetle_header = tuple((le_u16, le_u16, le_u16));
    let parse_res: IResult<_, _> = beetle_header(slice);

    let (rest, size) = match parse_res {
        Ok(r) => r,
        Err(_) => return Err("Parse error"),
    };

    let size = vector![size.0 as usize, size.1 as usize, size.2 as usize];

    let mapped: Vec<u16> = rest
        .chunks(2)
        .map(|x| {
            let arr = x.try_into().unwrap_or([0; 2]);
            let mut v = u16::from_le_bytes(arr);
            v &= 0b0000111111111111;
            v
        })
        .collect();

    let parsed_vb = ParsedVolumeBuilder {
        size,
        border: 0,
        scale: vector![1.0 * 0.99, 1.0 * 0.99, 1.0 * 0.99],
        data: DataSource::Vec(mapped),
    };

    Ok(parsed_vb)
}

// todo move parsers - maybe to user space
pub fn dat_parser(vb: VolumeBuilder) -> Result<ParsedVolumeBuilder<u16>, &'static str> {
    let slice = if let Some(ref mmap) = vb.mmap {
        &mmap[..]
    } else if let Some(ref vec) = vb.data {
        &vec[..]
    } else {
        return Err("No data in VolumeBuilder");
    };

    let x_bytes: [u8; 2] = slice[0..2].try_into().map_err(|_| "Metadata error")?;
    let x = u16::from_le_bytes(x_bytes) as usize;

    let y_bytes: [u8; 2] = slice[2..4].try_into().map_err(|_| "Metadata error")?;
    let y = u16::from_le_bytes(y_bytes) as usize;

    let z_bytes: [u8; 2] = slice[4..6].try_into().map_err(|_| "Metadata error")?;
    let z = u16::from_le_bytes(z_bytes) as usize;

    let mapped: Vec<u16> = slice[6..]
        .chunks(2)
        .map(|x| {
            let arr = x.try_into().unwrap_or([0; 2]);
            let mut v = u16::from_le_bytes(arr);
            v &= 0b0000111111111111;
            v
        })
        .collect();

    println!(
        "Parsed .dat file. voxels: {} | planes: {} | plane: {}x{} ZxY",
        mapped.len(),
        x,
        z,
        y
    );

    let parsed_vb = ParsedVolumeBuilder {
        size: vector![x, y, z],
        border: 0,
        scale: vector![1.0 * 0.99, 1.0 * 0.99, 1.0 * 0.99],
        data: DataSource::Vec(mapped),
    };

    Ok(parsed_vb)
}

pub fn skull_parser(vb: VolumeBuilder) -> Result<ParsedVolumeBuilder<u8>, &'static str> {
    let storage = if let Some(mmap) = vb.mmap {
        let tm: TypedMmap<u8> = TypedMmap::from_map(mmap);
        DataSource::Mmap(tm)
    } else if let Some(vec) = vb.data {
        DataSource::Vec(vec)
    } else {
        return Err("No data in VolumeBuilder");
    };

    let slice = match storage {
        DataSource::Vec(ref v) => &v[..],
        DataSource::Mmap(ref tm) => tm.get_all(),
        DataSource::None => return Err("No data in VolumeBuilder"),
    };

    let parse_res = skull_inner(slice);

    match parse_res {
        Ok((_sl, (size, scale))) => {
            let result = ParsedVolumeBuilder::<u8> {
                size,
                border: 0,
                scale,
                data: storage,
            };
            Ok(result)
        }
        Err(_) => Err("Parse error"),
    }
}

fn skull_inner(s: &[u8]) -> IResult<&[u8], (Vector3<usize>, Vector3<f32>)> {
    let mut skull_header = tuple((
        tuple((be_u32, be_u32, be_u32)),
        take(4_u8),
        tuple((be_f32, be_f32, be_f32)),
    ));

    let (s, (size, _, scale)) = skull_header(s)?;

    let size = vector![size.0 as usize, size.1 as usize, size.2 as usize];
    let scale = vector![scale.0 * 0.999, scale.1 * 0.999, scale.2 * 0.999];

    Ok((s, (size, scale)))
}
