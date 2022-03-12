use std::sync::{Arc, RwLock};

use crossbeam_channel::{Receiver, Sender};
use nalgebra::Vector3;

use crate::{
    camera::{Camera, PerspectiveCamera},
    common::ViewportBox,
    volumetric::Block,
};

use super::messages::{ToCompositorMsg, ToRendererMsg};

pub struct CompositorWorker<'a> {
    compositor_id: usize,
    camera: Arc<RwLock<PerspectiveCamera>>,
    area: ViewportBox,
    resolution: (usize, usize), // Resolution of full image
    renderers: [Sender<ToRendererMsg>; 4],
    receiver: Receiver<ToCompositorMsg>,
    blocks: &'a [Block],
}

impl<'a> CompositorWorker<'a> {
    pub fn new(
        compositor_id: usize,
        camera: Arc<RwLock<PerspectiveCamera>>,
        area: ViewportBox,
        resolution: (usize, usize),
        renderers: [Sender<ToRendererMsg>; 4],
        receiver: Receiver<ToCompositorMsg>,
        blocks: &'a [Block],
    ) -> Self {
        Self {
            compositor_id,
            camera,
            area,
            resolution,
            renderers,
            receiver,
            blocks,
        }
    }

    pub fn run(&self) {
        // Subcanvas
        let subcanvas_size = self.calc_resolution();
        let subcanvas_items = subcanvas_size.0 * subcanvas_size.1;
        let subcanvas_rgb = vec![Vector3::<f32>::zeros(); subcanvas_items]; // todo RGB
        let subcanvas_opacity = vec![0.0; subcanvas_items];

        // Calculate which subvolumes appear in my subcanvas
        // Also calculate expected order of subvolumes
        let relevant_ids: Vec<_> = {
            let camera = self.camera.read().unwrap();

            (0..self.blocks.len())
                .filter(|&i| self.is_in_subcanvas(&camera, &self.blocks[i]))
                .collect()
        };

        loop {
            // Receive requests

            // Send opacity / store subrender / finish

            // Finally convert to RGB bytes and send to master thread for output
        }
    }

    fn is_in_subcanvas(&self, camera: &PerspectiveCamera, block: &Block) -> bool {
        let viewport_box = camera.project_box(block.bound_box);
        viewport_box.crosses(self.area)
    }

    // Resolution of subcanvas
    fn calc_resolution(&self) -> (usize, usize) {
        // Should pad to one direction, to distribute bordering pixels
        // 0,0 -> 0,0 | 1,1 -> width-1,height-1
        todo!()
    }
}
