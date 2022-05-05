/*
    raycaster_lib
    Author: Michal Majer
    Date: 2022-05-05
*/

use nalgebra::{vector, Vector4};

pub type RGBA = Vector4<f32>;

pub fn new(r: f32, g: f32, b: f32, a: f32) -> RGBA {
    vector![r, g, b, a]
}

pub fn zero() -> RGBA {
    vector![0.0, 0.0, 0.0, 0.0]
}

pub fn mono(v: f32, opacity: f32) -> RGBA {
    vector![v, v, v, opacity]
}
