use super::{ParsedVolumeBuilder, VolumeBuilder};
use nalgebra::vector;

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
        scale: vector![1.0, 1.0, 1.0],
        data: Some(mapped),
        mmap: None,
    };

    Ok(parsed_vb)
}

pub fn vol_parser(vb: VolumeBuilder) -> Result<ParsedVolumeBuilder<u8>, &'static str> {
    let slice = if let Some(ref mmap) = vb.mmap {
        &mmap[..]
    } else if let Some(ref vec) = vb.data {
        &vec[..]
    } else {
        return Err("No data in VolumeBuilder");
    };

    let x_bytes: [u8; 4] = slice[0..4].try_into().map_err(|_| "Metadata error")?;
    let x = u32::from_be_bytes(x_bytes) as usize;

    let y_bytes: [u8; 4] = slice[4..8].try_into().map_err(|_| "Metadata error")?;
    let y = u32::from_be_bytes(y_bytes) as usize;

    let z_bytes: [u8; 4] = slice[8..12].try_into().map_err(|_| "Metadata error")?;
    let z = u32::from_be_bytes(z_bytes) as usize;

    // skip 4 bytes

    let xs_bytes: [u8; 4] = slice[16..20].try_into().map_err(|_| "Metadata error")?;
    let scale_x = f32::from_be_bytes(xs_bytes);

    let ys_bytes: [u8; 4] = slice[20..24].try_into().map_err(|_| "Metadata error")?;
    let scale_y = f32::from_be_bytes(ys_bytes);

    let zs_bytes: [u8; 4] = slice[24..28].try_into().map_err(|_| "Metadata error")?;
    let scale_z = f32::from_be_bytes(zs_bytes);

    println!("zs bytes {:?}", zs_bytes);

    println!(
        "Parsed .vol file. voxels: {} | planes: {} | plane: {}x{} ZxY | scale: {} {} {}",
        slice.len(),
        x,
        z,
        y,
        scale_x,
        scale_y,
        scale_z
    );

    let parsed_vb = ParsedVolumeBuilder {
        size: vector![x, y, z],
        border: 0,
        scale: vector![scale_x, scale_y, scale_z],
        data: None,
        mmap: vb.mmap,
    };

    Ok(parsed_vb)
}
