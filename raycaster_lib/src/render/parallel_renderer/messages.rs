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

/// Safety: subcanvas is read by at most one thread, 'ownership' is passed by channels
unsafe impl Send for RenderTask {}

pub enum ToWorkerMsg {
    GoIdle,
    GoLive, // Go to active state, mainly seize camera and recalc distances
    Finish,
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

pub enum ToMasterMsg {
    RenderDone,
}
