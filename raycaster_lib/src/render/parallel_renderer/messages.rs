use nalgebra::Vector3;

use crate::common::PixelBox;

use super::composition::SubCanvas;

// Todo combine Vec and pixelbox to newtype

pub struct RenderTask {
    pub block_id: usize,
    pub tile_id: usize,
    pub subcanvas: *mut SubCanvas,
}

impl RenderTask {
    pub fn new(block_id: usize, tile_id: usize, subcanvas: *mut SubCanvas) -> Self {
        Self {
            block_id,
            tile_id,
            subcanvas,
        }
    }
}

unsafe impl Send for RenderTask {}

pub enum ToWorkerMsg {
    StopRendering,
    GoIdle,
    GoLive, // Go to active state, mainly seize camera and recalc distances
    Finish,
}

#[derive(Clone, Copy)]
pub struct OpacityRequest {
    pub from_id: usize, // Id of renderer
    pub order: usize,   // order by distance from the camera
}

impl std::fmt::Debug for OpacityRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("OR ord {}", self.order))
    }
}

impl OpacityRequest {
    pub fn new(from_id: usize, order: usize) -> Self {
        Self { from_id, order }
    }
}

// todo split color and transmit it at lower priority
pub struct SubRenderResult {
    pub tile_id: usize,
}

impl SubRenderResult {
    pub fn new(tile_id: usize) -> Self {
        Self { tile_id }
    }
}

pub struct OpacityData {
    pub from_compositor: usize,
    pub pixels: PixelBox,
    pub opacities: Vec<f32>,
}

impl OpacityData {
    pub fn new(from_compositor: usize, pixels: PixelBox, opacities: Vec<f32>) -> Self {
        Self {
            from_compositor,
            pixels,
            opacities,
        }
    }
}

pub struct SubFrameResult {
    pub from_id: usize,
    pub data: Vec<u8>,
}

impl SubFrameResult {
    #[must_use]
    pub fn new(from_id: usize, data: Vec<u8>) -> Self {
        Self { from_id, data }
    }
}

pub enum ToCompositorMsg {
    OpacityRequest(OpacityRequest),
    RenderResult(SubRenderResult),
}

pub enum ToMasterMsg {
    RenderDone,
}

pub enum ToRendererMsg {
    Opacity(OpacityData),
    EmptyOpacity,
}
