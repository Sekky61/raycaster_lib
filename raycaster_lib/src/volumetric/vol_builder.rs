use std::{fs::File, marker::PhantomData, mem::size_of, path::Path};

use memmap::{Mmap, MmapOptions};
use nalgebra::Vector3;

use super::Volume;

pub trait BuildVolume<M>
where
    Self: Sized,
{
    fn build(metadata: M, data: DataSource<u8>) -> Result<Self, &'static str>;
}

pub fn from_file<P, T, M>(
    path: P,
    parser: fn(&[u8]) -> Result<M, &'static str>,
) -> Result<T, &'static str>
where
    P: AsRef<Path>,
    T: BuildVolume<M> + Volume,
{
    let ds: DataSource<u8> = DataSource::from_file(path)?;
    let slice = ds.get_slice().ok_or("Cannot get data")?;
    let metadata = parser(slice)?;
    BuildVolume::<M>::build(metadata, ds)
}

pub enum Endianness {
    Big,
    Little,
}

pub struct TypedMmap<T> {
    mmap: Mmap,
    endianness: Endianness,
    offset: usize,
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
            offset: 0,
        }
    }

    pub fn into_inner(self) -> (Mmap, usize) {
        (self.mmap, self.offset)
    }

    pub fn set_offset(&mut self, offset: usize) {
        self.offset = offset;
    }

    pub fn get_all(&self) -> &[T] {
        let s = &self.mmap[self.offset..];
        let slice =
            unsafe { std::slice::from_raw_parts(s.as_ptr() as *const T, s.len() / size_of::<T>()) };
        slice
    }

    pub fn get(&self, index: usize) -> T {
        let s = &self.mmap[self.offset..];
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

impl<T> DataSource<T>
where
    T: Copy,
{
    pub fn get_slice(&self) -> Option<&[T]> {
        match self {
            DataSource::Vec(v) => Some(v.as_slice()),
            DataSource::Mmap(m) => Some(m.get_all()),
            DataSource::None => None,
        }
    }

    pub fn from_mmap(mmap: Mmap) -> DataSource<T> {
        let typed_map = TypedMmap::from_map(mmap);
        DataSource::Mmap(typed_map)
    }

    pub fn from_vec(vec: Vec<T>) -> DataSource<T> {
        DataSource::Vec(vec)
    }

    pub fn from_file<P>(path: P) -> Result<DataSource<T>, &'static str>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();

        if !path.is_file() {
            return Err("Path does not lead to a file");
        }

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

        let data_source = DataSource::from_mmap(mmap);
        Ok(data_source)
    }
}

pub struct VolumeMetadata {
    pub size: Vector3<usize>,
    pub border: u32,
    pub scale: Vector3<f32>, // shape of voxels
    pub data_offset: usize,
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
