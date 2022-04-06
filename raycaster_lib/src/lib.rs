//! # Raycaster lib crate
//!
//! Rust library for volumetric raycasting
//!
//! # Example

// #![feature(generic_const_exprs)]

pub mod color;
mod common;
mod perspective_camera;
pub mod premade;
pub mod render;
pub mod test_helpers;
pub mod volumetric;

pub use perspective_camera::PerspectiveCamera;

use volumetric::RGBA;
pub type TF = fn(f32) -> RGBA; // todo tf module, with trait
