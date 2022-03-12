use std::sync::{Arc, RwLock};

use crossbeam_channel::{Receiver, Sender};
use nalgebra::Vector3;

use crate::{
    camera::{Camera, PerspectiveCamera},
    common::{PixelBox, ViewportBox},
    volumetric::Block,
};

use super::messages::{OpacityData, ToCompositorMsg, ToRendererMsg};

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
        let subcanvas_items = subcanvas_size.items();
        let subcanvas_rgb = vec![Vector3::<f32>::zeros(); subcanvas_items]; // todo RGB
        let subcanvas_opacity = vec![0.0; subcanvas_items];

        // Calculate info about blocks
        let block_info = self.get_block_info();

        // Calculate which subvolumes appear in my subcanvas
        // Indexes are in asc. order
        let relevant_blocks: Vec<usize> = block_info
            .iter()
            .enumerate()
            .filter(|(i, info)| self.area.crosses(&info.viewport))
            .map(|v| v.0)
            .collect();

        // Also calculate expected order of subvolumes
        let mut order_indexes = relevant_blocks.clone();
        order_indexes.sort_unstable_by(|&i, &j| {
            block_info[i]
                .distance
                .partial_cmp(&block_info[j].distance)
                .unwrap()
        });

        let expected_volume = 0; // pointer into order_indexes, todo peekable iter

        loop {
            // Receive requests
            let request = self.receiver.recv().unwrap();
            match request {
                ToCompositorMsg::OpacityRequest(req) => {
                    let responder = &self.renderers[req.from_id];

                    match relevant_blocks.binary_search_by(|item| item.cmp(&req.order)) {
                        Ok(index) => {
                            // Block is in compositors field

                            if order_indexes[expected_volume] == index {
                                // Block is up

                                let info = &block_info[index];
                                let box_intersection = self.area.intersection(&info.viewport);
                                let pixels = box_intersection.get_pixel_range(self.resolution);

                                let opacity_data = OpacityData::new(pixels, vec![]);
                                let response = ToRendererMsg::Opacity(opacity_data);

                                responder.send(response).unwrap();
                            } else {
                                // Needs to be placed in queue
                            }
                        }
                        Err(_) => {
                            // Block is not in compositors field
                            let response = ToRendererMsg::EmptyOpacity;
                            responder.send(response).unwrap();
                        }
                    }
                }
                ToCompositorMsg::RenderResult(res) => {
                    // Update opacity map if block is in order

                    // Update next expected volume

                    // Expected volume is updated, can we satisfy request from queue?

                    // Got all results? Convert to RGB bytes and send to master thread for output
                }
                ToCompositorMsg::Finish => return,
            }
        }
    }

    // Resolution of subcanvas
    fn calc_resolution(&self) -> PixelBox {
        // Should pad to one direction, to distribute bordering pixels
        // 0,0 -> 0,0 | 1,1 -> width-1,height-1
        self.area.get_pixel_range(self.resolution)
    }

    fn get_block_info(&self) -> Vec<BlockInfo> {
        let mut relevant_ids = vec![];
        {
            let camera = self.camera.read().unwrap();

            for block in self.blocks {
                let viewport = camera.project_box(block.bound_box);
                let distance = camera.box_distance(&block.bound_box);
                let info = BlockInfo::new(distance, viewport);
            }
        }
        relevant_ids
    }
}

pub struct BlockInfo {
    distance: f32,
    viewport: ViewportBox,
}

impl BlockInfo {
    pub fn new(distance: f32, viewport: ViewportBox) -> Self {
        Self { distance, viewport }
    }
}
