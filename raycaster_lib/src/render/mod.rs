mod frame_swap;
mod renderer;

pub use renderer::{RenderOptions, Renderer};

pub use frame_swap::BufferStatus;

use crate::Camera;
use crate::{ray::Ray, volumetric::Volume};
use nalgebra::{vector, Vector3};
