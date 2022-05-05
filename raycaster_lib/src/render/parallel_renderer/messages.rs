/*
    raycaster_lib
    Author: Michal Majer
    Date: 2022-05-05
*/

use super::composition::SubCanvas;

/// Structure describing rendering task.
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
// todo send reference to unsafecell, less unsafe code
/// Safety: subcanvas is read by at most one thread, 'ownership' is passed by channels
unsafe impl Send for RenderTask {}

/// Controling messages to worker threads: `RenderWorker` (RV) and `CompWorker` (KV).
pub enum ToWorkerMsg {
    GoIdle,
    /// Go to active state, mainly seize camera and recalc distances.
    GoLive {
        /// Step length along ray during color integration.
        sample_step: f32,
    },
    /// Wrap up, get ready to be joined.
    Finish,
}

/// Message telling `CompWorker`, that task on tile `tile_id` is done.
pub struct SubRenderResult {
    pub tile_id: u32,
}

impl SubRenderResult {
    pub fn new(tile_id: u32) -> Self {
        Self { tile_id }
    }
}

/// Messgae sent by `CompWorker` telling master render is done.
pub enum ToMasterMsg {
    RenderDone,
}
