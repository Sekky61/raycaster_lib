mod renderer;

mod mt_linear;
mod st_linear;

pub use renderer::{Renderer, RendererOptions};

use crate::Camera;
use crate::{
    ray::Ray,
    volumetric::{LinearVolume, Volume},
};
use nalgebra::{vector, Vector3, Vector4};
