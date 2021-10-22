mod renderer;

pub use renderer::{RenderOptions, Renderer};

use crate::Camera;
use crate::{ray::Ray, volumetric::Volume};
use nalgebra::{vector, Vector3, Vector4};
