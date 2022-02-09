use std::{fs::File, marker::PhantomData, mem::size_of, path::Path};

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

pub trait BuildVolume<T>
where
    Self: Sized,
{
    fn build(builder: T) -> Result<Self, &'static str>;
}

pub fn from_file<P, T, U>(
    path: P,
    parser: fn(VolumeBuilder) -> Result<U, &'static str>,
) -> Result<T, &'static str>
where
    P: AsRef<Path>,
    T: BuildVolume<U>,
{
    let vb = VolumeBuilder::from_file(path)?;
    let parse_res = parser(vb)?;
    BuildVolume::<U>::build(parse_res)
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

pub enum Endianness {
    Big,
    Little,
}

pub struct TypedMmap<T> {
    mmap: Mmap,
    endianness: Endianness,
    t: PhantomData<T>,
}

impl<T> TypedMmap<T>
where
    T: Copy,
{
    pub fn from_map(mmap: Mmap) -> TypedMmap<T> {
        TypedMmap::<T> {
            mmap,
            endianness: Endianness::Little,
            t: Default::default(),
        }
    }

    pub fn into_inner(self) -> Mmap {
        self.mmap
    }

    pub fn get_all(&self) -> &[T] {
        let s = &self.mmap[..];
        let slice =
            unsafe { std::slice::from_raw_parts(s.as_ptr() as *const T, s.len() / size_of::<T>()) };
        slice
    }

    pub fn get(&self, index: usize) -> T {
        let s = &self.mmap[..];
        let index = index * size_of::<T>();
        let slice =
            unsafe { std::slice::from_raw_parts(s.as_ptr() as *const T, s.len() / size_of::<T>()) };
        slice[index]
    }
}

pub enum DataSource<T> {
    Vec(Vec<T>),
    Mmap(TypedMmap<T>),
    None,
}

pub struct ParsedVolumeBuilder<T> {
    pub(super) size: Vector3<usize>,
    pub(super) border: u32,
    pub(super) scale: Vector3<f32>, // shape of voxels
    pub(super) data: DataSource<T>,
}

impl<T> ParsedVolumeBuilder<T>
where
    T: Default + Clone + Copy,
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
            DataSource::Vec(v) => v.get(index).cloned().unwrap_or_default(),
            DataSource::Mmap(m) => m.get(index),
            DataSource::None => Default::default(),
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
