use std::ops::Range;

use nalgebra::Vector3;

use crate::common::PixelBox;

pub struct RenderTask {
    pub block_order: usize,
}

// todo masterToRendererMsg with finish, camera_changed

impl RenderTask {
    pub fn new(block_order: usize) -> Self {
        Self { block_order }
    }
}

pub struct OpacityRequest {
    pub from_id: usize, // Id of renderer
    pub order: usize,   // order by distance from the camera
}

impl OpacityRequest {
    pub fn new(from_id: usize, order: usize) -> Self {
        Self { from_id, order }
    }
}

// todo split color and transmit it at lower priority
pub struct SubRenderResult {
    pub block_id: usize,
    pub width: usize,
    pub colors: Vec<Vector3<f32>>,
    pub opacities: Vec<f32>,
}

impl SubRenderResult {
    pub fn new(
        block_id: usize,
        width: usize,
        colors: Vec<Vector3<f32>>,
        opacities: Vec<f32>,
    ) -> Self {
        Self {
            block_id,
            width,
            colors,
            opacities,
        }
    }
}

pub struct OpacityData {
    pixels: PixelBox,
    opacities: Vec<f32>,
}

impl OpacityData {
    pub fn new(pixels: PixelBox, opacities: Vec<f32>) -> Self {
        Self { pixels, opacities }
    }
}

pub enum ToCompositorMsg {
    OpacityRequest(OpacityRequest),
    RenderResult(SubRenderResult),
    Finish,
}

pub enum ToRendererMsg {
    Opacity(OpacityData),
    EmptyOpacity,
}
