use std::{fs::File, mem::size_of, path::Path};

use memmap::{Mmap, MmapOptions};
use nalgebra::Vector3;

use super::{Volume, TF};

pub trait BuildVolume<M>
where
    Self: Sized,
{
    fn build(metadata: M, data: DataSource<u8>, tf: TF) -> Result<Self, &'static str>;
}

pub fn from_file<P, T, M>(
    path: P,
    parser: fn(&[u8]) -> Result<M, &'static str>,
    tf: TF,
) -> Result<T, &'static str>
where
    P: AsRef<Path>,
    T: BuildVolume<M> + Volume,
{
    let ds: DataSource<u8> = DataSource::from_file(path)?;
    let slice = ds.get_slice().ok_or("Cannot get data")?;
    let metadata = parser(slice)?;
    BuildVolume::<M>::build(metadata, ds, tf)
}

pub struct TypedMmap {
    mmap: Mmap,
    offset: usize,
}

impl TypedMmap {
    pub fn from_map(mmap: Mmap) -> TypedMmap {
        TypedMmap { mmap, offset: 0 }
    }

    pub fn into_inner(self) -> (Mmap, usize) {
        (self.mmap, self.offset)
    }

    pub fn set_offset(&mut self, offset: usize) {
        self.offset = offset;
    }

    pub fn get_all<T>(&self) -> &[T] {
        let s = &self.mmap[self.offset..];
        let slice =
            unsafe { std::slice::from_raw_parts(s.as_ptr() as *const T, s.len() / size_of::<T>()) };
        slice
    }

    pub fn get<T: Copy>(&self, index: usize) -> T {
        let s = &self.mmap[self.offset..];
        let index = index * size_of::<T>();
        let slice =
            unsafe { std::slice::from_raw_parts(s.as_ptr() as *const T, s.len() / size_of::<T>()) };
        slice[index]
    }

    pub fn get_ref<T>(&self, index: usize) -> &T {
        let s = &self.mmap[self.offset..];
        let index = index * size_of::<T>();
        let slice =
            unsafe { std::slice::from_raw_parts(s.as_ptr() as *const T, s.len() / size_of::<T>()) };
        &slice[index]
    }
}

pub enum DataSource<T> {
    Vec(Vec<T>),
    Mmap(TypedMmap),
    None,
}

impl<T> DataSource<T> {
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
