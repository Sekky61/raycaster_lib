mod renderer;

mod linear_render;

pub use renderer::{Renderer, RendererOptions};

use crate::Camera;
use crate::{
    ray::Ray,
    volumetric::{LinearVolume, Volume},
};
use nalgebra::{vector, Vector3, Vector4};
