use std::{fs::File, path::Path};

use memmap::{Mmap, MmapOptions};
use nalgebra::{vector, Vector3};

use super::Volume;

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
pub struct VolumeBuilder {
    pub(super) size: Vector3<usize>,
    pub(super) border: u32,
    pub(super) scale: Vector3<f32>, // shape of voxels
    pub(super) data: Vec<u8>,
    pub(super) mmap: Option<Mmap>,
}

pub trait BuildVolume {
    fn build(builder: VolumeBuilder) -> Self;
}

impl VolumeBuilder {
    pub fn new() -> VolumeBuilder {
        VolumeBuilder {
            size: vector![0, 0, 0],
            border: 0,
            scale: vector![1.0, 1.0, 1.0],
            data: vec![],
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

        match extension {
            "vol" => vol_parser(mmap),
            "dat" => dat_parser(mmap),
            _ => Err("Unknown extension"),
        }
    }

    pub fn set_size(mut self, size: Vector3<usize>) -> VolumeBuilder {
        self.size = size;
        self
    }

    pub fn set_mmap(mut self, mmap: Mmap) -> VolumeBuilder {
        self.mmap = Some(mmap);
        self
    }

    pub fn set_border(mut self, border: u32) -> VolumeBuilder {
        self.border = border;
        self
    }

    pub fn set_scale(mut self, scale: Vector3<f32>) -> VolumeBuilder {
        self.scale = scale;
        self
    }

    pub fn set_data(mut self, data: Vec<u8>) -> VolumeBuilder {
        self.data = data;
        self
    }

    pub fn build<V>(self) -> V
    where
        V: Volume + BuildVolume,
    {
        assert!(self.size.iter().all(|&dim| dim != 0));
        assert!(self.scale.iter().all(|&dim| dim != 0.0));
        assert_eq!(self.data.len(), self.size.fold(1, |acc, side| acc * side));

        V::build(self)
    }

    fn get_3d_index(&self, x: usize, y: usize, z: usize) -> usize {
        z + y * self.size.z + x * self.size.y * self.size.z
    }

    pub fn get_data(&self, x: usize, y: usize, z: usize) -> u8 {
        if x > self.size.x || y > self.size.y || z > self.size.z {
            return Default::default();
        }
        let index = self.get_3d_index(x, y, z);
        match self.data.get(index) {
            Some(&v) => v,
            None => Default::default(),
        }
    }

    pub fn get_surrounding_data(&self, x: usize, y: usize, z: usize) -> [u8; 8] {
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

fn dat_parser(map: Mmap) -> Result<VolumeBuilder, &'static str> {
    let slice = &map[..];

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

    let volume_builder = VolumeBuilder::new()
        .set_size(vector![x, y, z])
        .set_mmap(map)
        .set_border(0);

    Ok(volume_builder)
}

fn vol_parser(map: Mmap) -> Result<VolumeBuilder, &'static str> {
    let slice = &map[..];

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
        map.len(),
        x,
        z,
        y,
        scale_x,
        scale_y,
        scale_z
    );

    let volume_builder = VolumeBuilder::new()
        .set_size(vector![x, y, z])
        .set_scale(vector![scale_x, scale_y, scale_z])
        .set_mmap(map)
        .set_border(0);

    Ok(volume_builder)
}
