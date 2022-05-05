/*
    raycaster_lib
    Author: Michal Majer
    Date: 2022-05-05
*/

//! # Raycaster library crate
//!
//! Rust library for direct volumetric raycasting.
//!
//! Source code is part of the generated documentation and is available
//! upon clicking `source` in upper right corner.
//!
//! # Example

pub mod color;
pub mod common;
mod perspective_camera;
pub mod premade;
pub mod render;
pub mod test_helpers;
pub mod volumetric;

pub use perspective_camera::PerspectiveCamera;

use color::RGBA;

pub type TF = fn(f32) -> RGBA; // todo tf module, with trait

pub type ParserFn =
    fn(volumetric::DataSource<u8>) -> Result<volumetric::VolumeMetadata<u8>, &'static str>;
