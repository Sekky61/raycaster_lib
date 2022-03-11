use std::ops::Range;

use nalgebra::Vector3;

pub struct OpacityRequest {
    order: usize, // distance from the camera
    pixel_range: (Range<usize>, Range<usize>),
}

pub struct SubRenderResult {
    width: usize,
    colors: Vec<Vector3<f32>>,
    opacities: Vec<f32>,
}

pub struct OpacityData {
    start_pixel: usize, // offset of lowest pixel
    width: usize,
    opacities: Vec<f32>,
}

pub enum ToCompositorMsg {
    OpacityRequest(OpacityRequest),
    RenderResult(SubRenderResult),
    Finish,
}

pub struct ToRendererMsg {
    opacity: OpacityData,
}
