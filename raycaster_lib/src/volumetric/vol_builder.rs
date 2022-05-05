/*
    raycaster_lib
    Author: Michal Majer
    Date: 2022-05-05
*/

use std::{fs::File, mem::size_of, path::Path};

use crate::TF;

use memmap::{Mmap, MmapOptions};
use nalgebra::{Point3, Vector3};

// Build Volume this trait is defined on from the metadata object
// T is the type of sample
pub trait BuildVolume<T>
where
    Self: Sized,
{
    fn build(metadata: VolumeMetadata<T>) -> Result<Self, &'static str>;
}

#[derive(Debug, Clone, Copy)]
pub enum MemoryType {
    Stream,
    Ram,
}

#[derive(Debug)]
pub enum StorageShape {
    Linear,
    Z(u8),
}

#[derive(Default)]
pub struct VolumeMetadata<T> {
    // Shape
    pub position: Option<Point3<f32>>,
    pub size: Option<Vector3<usize>>,
    pub scale: Option<Vector3<f32>>, // Shape of cells
    // Data
    pub data: Option<DataSource<T>>,
    pub memory_type: Option<MemoryType>,
    pub data_shape: Option<StorageShape>,
    pub desired_data_shape: Option<StorageShape>,
    pub tf: Option<TF>, // Transfer function
}

impl<T> VolumeMetadata<T> {
    pub fn new<U>() -> VolumeMetadata<U>
    // todo ??
    where
        U: Default,
    {
        Default::default()
    }

    pub fn set_position(&mut self, position: Point3<f32>) -> &mut Self {
        self.position = Some(position);
        self
    }

    pub fn set_size(&mut self, size: Vector3<usize>) -> &mut Self {
        self.size = Some(size);
        self
    }

    pub fn set_scale(&mut self, scale: Vector3<f32>) -> &mut Self {
        self.scale = Some(scale);
        self
    }

    pub fn set_data(&mut self, data: DataSource<T>) -> &mut Self {
        self.data = Some(data);
        self
    }

    pub fn set_tf(&mut self, tf: TF) -> &mut Self {
        self.tf = Some(tf);
        self
    }

    pub fn set_memory_type(&mut self, memory_type: MemoryType) -> &mut Self {
        self.memory_type = Some(memory_type);
        self
    }

    /// What data actually looks like in file.
    pub fn set_data_shape(&mut self, data_shape: StorageShape) -> &mut Self {
        self.data_shape = Some(data_shape);
        self
    }

    /// This represents what the shape should be, not what the shape of data is.
    pub fn set_desired_data_shape(&mut self, desired_data_shape: StorageShape) -> &mut Self {
        self.desired_data_shape = Some(desired_data_shape);
        self
    }
}

#[derive(Debug)]
pub struct TypedMmap {
    mmap: Mmap,
    offset: usize,
}

impl TypedMmap {
    pub fn from_map(mmap: Mmap) -> TypedMmap {
        TypedMmap { mmap, offset: 0 }
    }

    pub fn as_ptr<T>(&self) -> *const T {
        let ptr = self.mmap.as_ptr() as *const T;
        unsafe { ptr.add(self.offset) }
    }

    pub fn into_inner(self) -> (Mmap, usize) {
        (self.mmap, self.offset)
    }

    pub fn set_offset(&mut self, offset: usize) {
        self.offset = offset;
    }

    pub fn get_raw(&self) -> &[u8] {
        &self.mmap[self.offset..]
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

#[derive(Debug)]
pub enum DataSource<T> {
    Vec(Vec<T>),
    Mmap(TypedMmap),
}

impl<T: Clone> DataSource<T> {
    pub fn into<U>(self) -> DataSource<U>
    where
        T: Into<U> + Copy,
    {
        match self {
            DataSource::Vec(ref v) => {
                let new = v.iter().map(|&v| v.into()).collect();
                DataSource::Vec(new)
            }
            DataSource::Mmap(m) => DataSource::Mmap(m),
        }
    }

    pub fn as_ptr(&self) -> *const T {
        match self {
            DataSource::Vec(v) => v.as_ptr(),
            DataSource::Mmap(m) => m.as_ptr(),
        }
    }

    /// Adds offset
    /// Offset in elements, not bytes
    pub fn clone_with_offset(self, offset: usize) -> Self {
        match self {
            DataSource::Vec(v) => {
                let slice = &v.as_slice()[offset..];
                let new_vec = slice.to_owned();
                DataSource::Vec(new_vec)
            }
            DataSource::Mmap(mut m) => {
                m.offset += offset;
                DataSource::Mmap(m)
            }
        }
    }

    pub fn into_transmute<U>(self) -> DataSource<U> {
        match self {
            DataSource::Vec(v) => {
                let ptr = v.as_ptr();
                let elements = v.len();
                let allocated_elements = v.capacity();
                //let (ptr, elements, allocated_elements) = v.into_raw_parts();
                let growth = size_of::<T>() / size_of::<U>();
                let new_length = elements * growth;
                let new_allocated = allocated_elements * growth;
                let new = unsafe { Vec::from_raw_parts(ptr as *mut U, new_length, new_allocated) };
                DataSource::Vec(new)
            }
            DataSource::Mmap(m) => DataSource::Mmap(m),
        }
    }

    pub fn get_slice_transmute<U>(&self) -> &[U] {
        let slice = match self {
            DataSource::Vec(v) => v.as_slice(),
            DataSource::Mmap(m) => m.get_all(),
        };

        let ptr = slice.as_ptr();
        let len = slice.len();
        let growth = size_of::<T>() / size_of::<U>();
        let new_length = len * growth;

        unsafe { std::slice::from_raw_parts(ptr as *mut U, new_length) }
    }

    pub fn get_slice(&self) -> &[T] {
        match self {
            DataSource::Vec(v) => v.as_slice(),
            DataSource::Mmap(m) => m.get_all(),
        }
    }

    pub fn get(&self, index: usize) -> Option<T> {
        self.get_slice().get(index).cloned()
    }

    pub fn from_mmap(mmap: Mmap) -> DataSource<T> {
        let typed_map = TypedMmap::from_map(mmap);
        DataSource::Mmap(typed_map)
    }

    pub fn from_vec(vec: Vec<T>) -> DataSource<T> {
        DataSource::Vec(vec)
    }

    /// Copy mmapped file to ram
    pub fn to_vec(self) -> Self {
        match self {
            DataSource::Vec(_) => self,
            DataSource::Mmap(m) => {
                let src = m.get_all();
                let vec = src.to_owned();
                DataSource::Vec(vec)
            }
        }
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
