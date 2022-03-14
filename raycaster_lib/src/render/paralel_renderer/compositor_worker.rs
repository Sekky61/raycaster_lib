use std::{
    ops::Range,
    sync::{Arc, RwLock},
};

use crossbeam_channel::{Receiver, Sender};
use nalgebra::{Vector2, Vector3};

use crate::{
    camera::{Camera, PerspectiveCamera},
    common::{PixelBox, ViewportBox},
    volumetric::Block,
};

use super::messages::{OpacityData, SubRenderResult, ToCompositorMsg, ToRendererMsg};

pub struct CompositorWorker<'a> {
    compositor_id: usize,
    camera: Arc<RwLock<PerspectiveCamera>>,
    area: ViewportBox,
    resolution: Vector2<usize>, // Resolution of full image
    renderers: [Sender<ToRendererMsg>; 4],
    receiver: Receiver<ToCompositorMsg>,
    blocks: &'a [Block],
}

impl<'a> CompositorWorker<'a> {
    pub fn new(
        compositor_id: usize,
        camera: Arc<RwLock<PerspectiveCamera>>,
        area: ViewportBox,
        resolution: Vector2<usize>,
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
        let mut subcanvas_rgb = vec![Vector3::<f32>::zeros(); subcanvas_items]; // todo RGB
        let mut subcanvas_opacity = vec![0.0; subcanvas_items];

        // Calculate info about blocks
        // Only subvolumes that appear in my subcanvas
        let block_info = self.get_block_info();

        let mut expected_volume = 0; // pointer into block_info, todo peekable iter

        loop {
            // Receive requests
            let request = self.receiver.recv().unwrap();
            match request {
                ToCompositorMsg::OpacityRequest(req) => {
                    let responder = &self.renderers[req.from_id];

                    match block_info.binary_search_by(|item| item.index.cmp(&req.order)) {
                        Ok(index) => {
                            // Block is in compositors field

                            if block_info[expected_volume].index == index {
                                // Block is up

                                let info = &block_info[index];
                                let box_intersection =
                                    self.area.intersection_unchecked(&info.viewport);
                                let pixels = box_intersection.get_pixel_range(self.resolution);

                                // Shift pixelbox to our subframe

                                let opacity_data = self.copy_opacity(
                                    subcanvas_opacity.as_slice(),
                                    &subcanvas_size,
                                    &pixels,
                                );
                                let pixel_range = PixelBox::new(todo!(), todo!());
                                let opacity_data = OpacityData::new(pixel_range, opacity_data);
                                let response = ToRendererMsg::Opacity(opacity_data);

                                responder.send(response).unwrap();
                            } else {
                                // Needs to be placed in queue
                                todo!()
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
                    // SubRenderResult buffers have the same shape as sent in their previous request

                    if block_info[expected_volume].index == res.block_id {
                        // Block is expected

                        // Update opacity
                        self.add_opacity(&mut subcanvas_opacity[..], &res);

                        // Update color

                        // Update next expected volume
                        expected_volume += 1;
                    } else {
                        // Not expected
                        // Renderer is sending only nonempty results
                        // and can only render what we allowed in OpacityRequest
                        panic!("Got RenderResult that is not expected - should not happen");
                    }

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

    // Return collection of blocks in the subframe
    // Collection is sorted by distance (asc.)
    fn get_block_info(&self) -> Vec<BlockInfo> {
        let mut relevant_ids = vec![];
        {
            let camera = self.camera.read().unwrap();

            for (i, block) in self.blocks.iter().enumerate() {
                let viewport = camera.project_box(block.bound_box);
                let distance = camera.box_distance(&block.bound_box);
                let info = BlockInfo::new(i, distance, viewport);
                if self.area.crosses(&info.viewport) {
                    relevant_ids.push(info);
                }
            }
        }
        relevant_ids.sort_unstable_by(|b1, b2| b1.distance.partial_cmp(&b2.distance).unwrap());

        relevant_ids
    }

    fn copy_opacity(&self, opacity: &[f32], subframe: &PixelBox, pixels: &PixelBox) -> Vec<f32> {
        let mut v = Vec::with_capacity(pixels.items());
        let width = pixels.x.end - pixels.x.start;
        let height = pixels.y.end - pixels.y.start;
        let subframe_width = subframe.x.end - subframe.x.start;

        let subframe_offset_x = pixels.x.start - subframe.x.start;
        let subframe_offset_y = pixels.y.start - subframe.y.start;

        let mut line_start = subframe_offset_y * subframe_width + subframe_offset_x;
        for _ in 0..height {
            v.extend(&opacity[line_start..line_start + width]);
            line_start += subframe_width;
        }
        v
    }

    fn add_opacity(&self, subcanvas_opacity: &mut [f32], res: &SubRenderResult) {
        let width = res.width;
        let opacities = &res.opacities[..];

        todo!()
    }
}

pub struct BlockInfo {
    index: usize,
    distance: f32,
    viewport: ViewportBox,
}

impl BlockInfo {
    pub fn new(index: usize, distance: f32, viewport: ViewportBox) -> Self {
        Self {
            index,
            distance,
            viewport,
        }
    }
}
