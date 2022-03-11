use std::sync::{Arc, RwLock};

use crossbeam_channel::{Receiver, Sender};
use nalgebra::Vector3;

use crate::{camera::PerspectiveCamera, volumetric::Block};

use super::messages::{ToCompositorMsg, ToRendererMsg};

pub struct CompositorWorker<'a> {
    compositor_id: usize,
    camera: Arc<RwLock<PerspectiveCamera>>,
    resolution: (usize, usize),
    renderers: [Sender<ToRendererMsg>; 4],
    receiver: Receiver<ToCompositorMsg>,
    blocks: &'a [Block],
}

impl<'a> CompositorWorker<'a> {
    pub fn new(
        compositor_id: usize,
        camera: Arc<RwLock<PerspectiveCamera>>,
        resolution: (usize, usize),
        renderers: [Sender<ToRendererMsg>; 4],
        receiver: Receiver<ToCompositorMsg>,
        blocks: &'a [Block],
    ) -> Self {
        Self {
            compositor_id,
            camera,
            resolution,
            renderers,
            receiver,
            blocks,
        }
    }

    pub fn run(&self) {
        // Subcanvas
        let subcanvas_size = (0, 0);
        let subcanvas_items = subcanvas_size.0 * subcanvas_size.1;
        let subcanvas_rgb = vec![Vector3::<f32>::zeros(); subcanvas_items]; // todo RGB
        let subcanvas_opacity = vec![0.0; subcanvas_items];

        // Calculate which subvolumes appear in my subcanvas
        // Also calculate expected order of subvolumes

        loop {
            // Receive requests

            // Send opacity / store subrender / finish

            // Finally convert to RGB bytes and send to master thread for output

            // Wait for wakeup call or finish call
        }
    }
}
