use std::convert::TryInto;
use std::path::Path;
use std::{fs::File, io::Read};

use nalgebra::vector;

use super::vol_builder::{color, RGBA};
use super::VolumeBuilder;

pub fn from_file<P>(path: P) -> Result<VolumeBuilder, &'static str>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();

    if !path.is_file() {
        return Err("Path does not lead to a file");
    }

    let extension = match path.extension() {
        Some(ext) => ext,
        None => return Err("File has no extension"),
    };

    let extension = extension.to_str().expect("error converting extension");

    match extension {
        "vol" => vol_parser(path),
        "dat" => dat_parser(path),
        _ => Err("Unknown extension"),
    }
}

fn dat_parser(path: &Path) -> Result<VolumeBuilder, &'static str> {
    let metadata = path.metadata();
    let metadata = match metadata {
        Ok(metadata) => metadata,
        Err(_) => return Err("No file metadata"),
    };

    let file_len = metadata.len() as usize;
    let mut fb = Vec::with_capacity(file_len);
    let mut file = File::open(path).expect("no file");
    // .read_to_end(&mut fb)
    // .expect("cannot read");

    // nom assert_eq!(parser("(abc)"), Ok(("", "abc")));

    let mut x_bytes: [u8; 2] = [0; 2];
    file.read_exact(&mut x_bytes[..]).expect("no bytes x");
    let x = u16::from_le_bytes(x_bytes);

    let mut y_bytes: [u8; 2] = [0; 2];
    file.read_exact(&mut y_bytes[..]).expect("no bytes y");
    let y = u16::from_le_bytes(y_bytes);

    let mut z_bytes: [u8; 2] = [0; 2];
    file.read_exact(&mut z_bytes[..]).expect("no bytes z");
    let z = u16::from_le_bytes(z_bytes);

    file.read_to_end(&mut fb).expect("Read to end fail");

    let x = x as usize;
    let y = y as usize;
    let z = z as usize;

    let mapped: Vec<u16> = fb
        .chunks(2)
        .map(|x| {
            let arr = x.try_into().unwrap_or([0; 2]);
            let mut v = u16::from_le_bytes(arr);
            v &= 0b0000111111111111;
            v
        })
        .collect();

    // println!(
    //     "VV: {:?}",
    //     mapped.iter().enumerate().filter(|(i, &v)| { v != 0 })
    // );
    // println!("VV: {:?}", &mapped[0..124000]);

    println!(
        "Parsed .dat file. voxels: {} | planes: {} | plane: {}x{} ZxY",
        mapped.len(),
        x,
        z,
        y
    );

    let volume_builder = VolumeBuilder::new()
        .set_size(vector![x, y, z])
        .set_data(mapped, beetle_transfer_function)
        .set_border(0);

    Ok(volume_builder)
}

fn vol_parser(path: &Path) -> Result<VolumeBuilder, &'static str> {
    let metadata = path.metadata();
    let metadata = match metadata {
        Ok(metadata) => metadata,
        Err(_) => return Err("No file metadata"),
    };

    let mut fb = Vec::with_capacity(metadata.len() as usize);
    File::open(path)
        .expect("no file")
        .read_to_end(&mut fb)
        .expect("cannot read");

    // nom assert_eq!(parser("(abc)"), Ok(("", "abc")));

    let x_bytes: [u8; 4] = fb.get(0..4).expect("no bytes x").try_into().expect("wrong");
    let x = u32::from_be_bytes(x_bytes);

    let y_bytes: [u8; 4] = fb.get(4..8).expect("no bytes y").try_into().expect("wrong");
    let y = u32::from_be_bytes(y_bytes);

    let z_bytes: [u8; 4] = fb
        .get(8..12)
        .expect("no bytes z")
        .try_into()
        .expect("wrong");
    let z = u32::from_be_bytes(z_bytes);

    // skip 4 bytes

    let xs_bytes: [u8; 4] = fb
        .get(16..20)
        .expect("no bytes x scale")
        .try_into()
        .expect("wrong");
    let scale_x = f32::from_be_bytes(xs_bytes);

    let ys_bytes: [u8; 4] = fb
        .get(20..24)
        .expect("no bytes y scale")
        .try_into()
        .expect("wrong");
    let scale_y = f32::from_be_bytes(ys_bytes);

    let zs_bytes: [u8; 4] = fb
        .get(24..28)
        .expect("no bytes z scale")
        .try_into()
        .expect("wrong");
    let scale_z = f32::from_be_bytes(zs_bytes);

    let rest = &fb[28..];

    println!(
        "Parsed .vol file. voxels: {} | planes: {} | plane: {}x{} ZxY | scale: {} {} {}",
        rest.len(),
        x,
        z,
        y,
        scale_x,
        scale_y,
        scale_z
    );

    let data = rest.to_owned();

    let x = x as usize;
    let y = y as usize;
    let z = z as usize;

    let volume_builder = VolumeBuilder::new()
        .set_size(vector![x, y, z])
        .set_scale(vector![scale_x, scale_y, scale_z])
        .set_data(data, skull_transfer_function)
        .set_border(0);

    Ok(volume_builder)
}

// R G B A -- A <0;1>
pub fn skull_transfer_function(sample: u8) -> RGBA {
    if sample > 170 {
        RGBA::new(220.0, 0.0, 20.0, 0.1)
    } else if sample > 130 {
        RGBA::new(0.0, 220.0, 0.0, 0.04)
    } else {
        color::zero()
    }
}

// R G B A -- A <0;1>
pub fn c60large_transfer_function(sample: u8) -> RGBA {
    if sample > 230 && sample < 255 {
        RGBA::new(200.0, 0.0, 0.0, 0.5)
    } else if sample > 200 && sample < 230 {
        RGBA::new(0.0, 180.0, 0.0, 0.3)
    } else if sample > 80 && sample < 120 {
        RGBA::new(2.0, 2.0, 60.0, 0.02)
    } else {
        color::zero()
    }
}

// R G B A -- A <0;1>
// uses just 12 bits
pub fn beetle_transfer_function(sample: u16) -> RGBA {
    if sample > 10000 {
        RGBA::new(255.0, 0.0, 0.0, 0.01)
    } else if sample > 5000 {
        RGBA::new(0.0, 255.0, 0.0, 0.01)
    } else if sample > 1900 {
        RGBA::new(0.0, 0.0, 255.0, 0.01)
    } else if sample > 800 {
        RGBA::new(10.0, 10.0, 10.0, 0.01)
    } else {
        color::zero()
    }
}
