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
    pub data_offset: Option<usize>,
    pub data_shape: Option<StorageShape>,
    pub tf: Option<TF>, // Transfer function
    pub block_side: Option<usize>,
}

impl<T> VolumeMetadata<T> {
    pub fn new<U>() -> VolumeMetadata<U>
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

    pub fn set_data_offset(&mut self, data_offset: usize) -> &mut Self {
        self.data_offset = Some(data_offset);
        self
    }

    pub fn set_tf(&mut self, tf: TF) -> &mut Self {
        self.tf = Some(tf);
        self
    }

    pub fn set_block_side(&mut self, block_side: usize) -> &mut Self {
        self.block_side = Some(block_side);
        self
    }
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
    pub fn new() -> DataSource<T> {
        DataSource::None
    }

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
            DataSource::None => DataSource::None,
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
            DataSource::None => DataSource::None,
        }
    }

    pub fn get_slice_transmute<U>(&self) -> Option<&[U]> {
        let slice = match self {
            DataSource::Vec(v) => Some(v.as_slice()),
            DataSource::Mmap(m) => Some(m.get_all()),
            DataSource::None => None,
        }?;

        let ptr = slice.as_ptr();
        let len = slice.len();
        let growth = size_of::<T>() / size_of::<U>();
        let new_length = len * growth;
        Some(unsafe { std::slice::from_raw_parts(ptr as *mut U, new_length) })
    }

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

impl<T> Default for DataSource<T> {
    fn default() -> Self {
        Self::new()
    }
}
