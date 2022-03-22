use nalgebra::Vector3;

use crate::common::PixelBox;

// Todo combine Vec and pixelbox to newtype

pub struct RenderTask {
    pub block_order: usize,
}

// todo masterToRendererMsg with finish, camera_changed

impl RenderTask {
    pub fn new(block_order: usize) -> Self {
        Self { block_order }
    }
}

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
    // todo remove pixels field, use order
    pub order: usize,
    pub pixels: PixelBox,
    pub colors: Vec<Vector3<f32>>,
    pub opacities: Vec<f32>,
}

impl SubRenderResult {
    pub fn new(
        order: usize,
        pixels: PixelBox,
        colors: Vec<Vector3<f32>>,
        opacities: Vec<f32>,
    ) -> Self {
        Self {
            order,
            pixels,
            colors,
            opacities,
        }
    }

    pub fn with_capacity(
        order: usize,
        pixels: PixelBox,
        capacity: usize,
        opacities: Vec<f32>,
    ) -> Self {
        let colors = Vec::with_capacity(capacity);
        Self {
            order,
            pixels,
            colors,
            opacities,
        }
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
    // todo remove offset and width, use from_id field
    pub data: Vec<u8>,
    pub offset: usize,
    pub width: usize,
}

impl SubFrameResult {
    pub fn new(data: Vec<u8>, offset: usize, width: usize) -> Self {
        Self {
            data,
            offset,
            width,
        }
    }
}

pub enum ToCompositorMsg {
    OpacityRequest(OpacityRequest),
    RenderResult(SubRenderResult),
}

pub enum ToMasterMsg {
    Subframe(SubFrameResult),
}

pub enum ToRendererMsg {
    Opacity(OpacityData),
    EmptyOpacity,
}
