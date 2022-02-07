use std::{fs::File, path::Path};

use memmap::{Mmap, MmapOptions};
use nalgebra::Vector3;

// pub(super) -- fields visible in parent module
// todo merge mmap and data to enum storage
#[derive(Default)]
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
        match &self.data {
            Some(vec) => vec.get(index).cloned().unwrap_or_default(),
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
