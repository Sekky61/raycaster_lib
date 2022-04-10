use super::composition::SubCanvas;

// Todo combine Vec and pixelbox to newtype

pub struct RenderTask {
    pub block_id: u32,
    pub tile_id: u32,
    pub subcanvas: *mut SubCanvas,
}

impl RenderTask {
    pub fn new(block_id: u32, tile_id: u32, subcanvas: *mut SubCanvas) -> Self {
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
    GoLive { quality: bool }, // Go to active state, mainly seize camera and recalc distances
    Finish,
}

// todo split color and transmit it at lower priority
pub struct SubRenderResult {
    pub tile_id: u32,
}

impl SubRenderResult {
    pub fn new(tile_id: u32) -> Self {
        Self { tile_id }
    }
}

pub enum ToMasterMsg {
    RenderDone,
}
