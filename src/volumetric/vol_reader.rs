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

    let mut fb = Vec::with_capacity(metadata.len() as usize);
    let mut file = File::open(path).expect("no file");
    // .read_to_end(&mut fb)
    // .expect("cannot read");

    // nom assert_eq!(parser("(abc)"), Ok(("", "abc")));

    let mut x_bytes: [u8; 2] = [0; 2];
    file.read_exact(&mut x_bytes[..]).expect("no bytes x");
    let x = u16::from_be_bytes(x_bytes);

    let mut y_bytes: [u8; 2] = [0; 2];
    file.read_exact(&mut y_bytes[..]).expect("no bytes y");
    let y = u16::from_be_bytes(y_bytes);

    let mut z_bytes: [u8; 2] = [0; 2];
    file.read_exact(&mut z_bytes[..]).expect("no bytes z");
    let z = u16::from_be_bytes(z_bytes);

    file.read_to_end(&mut fb).expect("Read to end fail");

    println!(
        "Parsed file. Rest: {} | Rest / 68 = {} | Rest / 256*256 = {}",
        fb.len(),
        fb.len() / x as usize,
        fb.len() / (y * z) as usize
    );

    let x = x as usize;
    let y = y as usize;
    let z = z as usize;

    let volume_builder = VolumeBuilder::new()
        .set_size(vector![x, y, z])
        .set_data(fb, transfer_function)
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
        "Parsed file. voxels: {} | planes: {} | plane: {}x{} ZxY | scale: {} {} {}",
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
        //.set_scale(vector![scale_x, scale_y, scale_z])
        .set_data(data, transfer_function)
        .set_border(0);

    Ok(volume_builder)
}

// R G B A -- A <0;1>
pub fn transfer_function(sample: f32) -> RGBA {
    if sample > 180.0 {
        RGBA::new(60.0, 230.0, 40.0, 0.3)
    } else if sample > 70.0 {
        RGBA::new(230.0, 10.0, 10.0, 0.3)
    } else if sample > 50.0 {
        RGBA::new(10.0, 20.0, 100.0, 0.1)
    } else if sample > 5.0 {
        RGBA::new(10.0, 10.0, 40.0, 0.05)
    } else {
        color::zero()
    }
}
