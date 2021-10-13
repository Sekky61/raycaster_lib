use nalgebra::{vector, Vector3};

use super::Volume;

// pub(super) -- fields visible in parent module
pub struct VolumeBuilder {
    pub(super) size: Vector3<usize>,
    pub(super) border: u32,
    pub(super) scale: Vector3<f32>, // shape of voxels
    pub(super) data: Vec<u8>,
}

impl VolumeBuilder {
    pub fn new() -> VolumeBuilder {
        VolumeBuilder {
            size: vector![0, 0, 0],
            border: 0,
            scale: vector![1.0, 1.0, 1.0],
            data: vec![],
        }
    }

    pub fn white_vol() -> VolumeBuilder {
        VolumeBuilder {
            size: vector![2, 2, 2],
            border: 0,
            scale: vector![100.0, 100.0, 100.0], // shape of voxels
            data: vec![0, 32, 64, 64 + 32, 128, 128 + 32, 128 + 64, 255],
        }
    }

    pub fn set_size(mut self, size: Vector3<usize>) -> VolumeBuilder {
        self.size = size;
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

    pub fn build<V: Volume + From<VolumeBuilder>>(self) -> V {
        let vol_dims = self.size.cast::<f32>().component_mul(&self.scale);

        V::from(self)
    }
}

impl Default for VolumeBuilder {
    fn default() -> Self {
        Self::new()
    }
}
