use std::{fs::File, path::Path};

use memmap::{Mmap, MmapOptions};
use nalgebra::{vector, Vector3};

use nalgebra::Vector4;

pub type RGBA = Vector4<f32>;

pub mod color {
    use super::*;

    pub fn new(r: f32, g: f32, b: f32, a: f32) -> RGBA {
        vector![r, g, b, a]
    }

    pub fn zero() -> RGBA {
        vector![0.0, 0.0, 0.0, 0.0]
    }

    pub fn mono(v: f32) -> RGBA {
        vector![v, v, v, v]
    }
}

// pub(super) -- fields visible in parent module
// todo merge mmap and data to enum storage
pub struct VolumeBuilder {
    pub(super) data: Option<Vec<u8>>,
    pub(super) file_ext: Option<String>,
    pub(super) mmap: Option<Mmap>,
}

pub trait BuildVolume<T> {
    fn build(builder: T) -> Self;
}

impl VolumeBuilder {
    pub fn new() -> VolumeBuilder {
        VolumeBuilder {
            data: None,
            file_ext: None,
            mmap: None,
        }
    }

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

        let file = File::open(path);

        let file = match file {
            Ok(f) => f,
            Err(_) => return Err("Cannot open file"),
        };

        let mmap = unsafe { MmapOptions::new().map(&file) };
        let mmap = match mmap {
            Ok(mmap) => mmap,
            Err(_) => return Err("Cannot create memory map"),
        };

        let vb = VolumeBuilder::new().set_extension(extension).set_mmap(mmap);
        Ok(vb)
    }

    pub fn set_mmap(mut self, mmap: Mmap) -> VolumeBuilder {
        self.mmap = Some(mmap);
        self
    }

    pub fn set_data(mut self, data: Vec<u8>) -> VolumeBuilder {
        self.data = Some(data);
        self
    }

    pub fn set_extension(mut self, ext: &str) -> VolumeBuilder {
        self.file_ext = Some(ext.into());
        self
    }
}

pub struct ParsedVolumeBuilder<T> {
    pub(super) size: Vector3<usize>,
    pub(super) border: u32,
    pub(super) scale: Vector3<f32>, // shape of voxels
    pub(super) data: Option<Vec<T>>,
    pub(super) mmap: Option<Mmap>,
}

impl<T> ParsedVolumeBuilder<T>
where
    T: Default + Clone,
{
    fn get_3d_index(&self, x: usize, y: usize, z: usize) -> usize {
        z + y * self.size.z + x * self.size.y * self.size.z
    }

    // from data vector only
    pub fn get_data(&self, x: usize, y: usize, z: usize) -> T {
        if x > self.size.x || y > self.size.y || z > self.size.z {
            return Default::default();
        }
        let index = self.get_3d_index(x, y, z);
        match self.data {
            Some(vec) => vec.get(index).cloned().unwrap_or(Default::default()),
            None => Default::default(),
        }
    }

    pub fn get_surrounding_data(&self, x: usize, y: usize, z: usize) -> [T; 8] {
        [
            self.get_data(x, y, z),
            self.get_data(x, y, z + 1),
            self.get_data(x, y + 1, z),
            self.get_data(x, y + 1, z + 1),
            self.get_data(x + 1, y, z),
            self.get_data(x + 1, y, z + 1),
            self.get_data(x + 1, y + 1, z),
            self.get_data(x + 1, y + 1, z + 1),
        ]
    }
}

// todo move parsers - maybe to user space
pub fn dat_parser(vb: VolumeBuilder) -> Result<ParsedVolumeBuilder<u16>, &'static str> {
    let slice = if let Some(mmap) = vb.mmap {
        &mmap[..]
    } else if let Some(vec) = vb.data {
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
    let slice = if let Some(mmap) = vb.mmap {
        &mmap[..]
    } else if let Some(vec) = vb.data {
        &vec[..]
    } else {
        return Err("No data in VolumeBuilder");
    };

    let x_bytes: [u8; 4] = slice[0..4].try_into().map_err(|_| "Metadata error")?;
    let x = u32::from_le_bytes(x_bytes) as usize;

    let y_bytes: [u8; 4] = slice[4..8].try_into().map_err(|_| "Metadata error")?;
    let y = u32::from_le_bytes(y_bytes) as usize;

    let z_bytes: [u8; 4] = slice[8..12].try_into().map_err(|_| "Metadata error")?;
    let z = u32::from_le_bytes(z_bytes) as usize;

    // skip 4 bytes

    let xs_bytes: [u8; 4] = slice[16..20].try_into().map_err(|_| "Metadata error")?;
    let scale_x = u32::from_le_bytes(xs_bytes) as f32;

    let ys_bytes: [u8; 4] = slice[20..24].try_into().map_err(|_| "Metadata error")?;
    let scale_y = u32::from_le_bytes(ys_bytes) as f32;

    let zs_bytes: [u8; 4] = slice[24..28].try_into().map_err(|_| "Metadata error")?;
    let scale_z = u32::from_le_bytes(zs_bytes) as f32;

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
