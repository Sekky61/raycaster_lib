use std::{
    collections::VecDeque,
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

use super::messages::{OpacityData, SubRenderResult, ToCompositorMsg, ToMasterMsg, ToRendererMsg};

pub struct CompositorWorker<'a> {
    compositor_id: usize,
    camera: Arc<RwLock<PerspectiveCamera>>,
    area: ViewportBox,
    resolution: Vector2<usize>, // Resolution of full image
    renderers: [Sender<ToRendererMsg>; 4],
    receiver: Receiver<ToCompositorMsg>,
    result_sender: Sender<ToMasterMsg>,
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
        result_sender: Sender<ToMasterMsg>,
        blocks: &'a [Block],
    ) -> Self {
        Self {
            compositor_id,
            camera,
            area,
            resolution,
            renderers,
            receiver,
            result_sender,
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
        let mut queue = VecDeque::new();

        loop {
            // Receive requests
            let request = self.receiver.recv().unwrap();
            match request {
                ToCompositorMsg::OpacityRequest(req) => {
                    let responder = &self.renderers[req.from_id];

                    match block_info.binary_search_by(|item| item.index.cmp(&req.order)) {
                        Ok(index) => {
                            // Block is in compositors field

                            // todo move forward, to skip binary search
                            if expected_volume == index {
                                // Block is up

                                let info = &block_info[index];

                                let response = self.handle_request(
                                    info,
                                    &subcanvas_opacity[..],
                                    &subcanvas_size,
                                );

                                responder.send(response).unwrap();
                            } else {
                                // Needs to be placed in queue
                                queue.push_back(req);
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
                        self.copy_opacity(&mut subcanvas_opacity[..], &subcanvas_size, &res);

                        // Update color
                        self.add_color(&mut subcanvas_rgb[..], &subcanvas_size, &res);

                        // Update next expected volume
                        expected_volume += 1;
                    } else {
                        // Not expected
                        // Renderer is sending only nonempty results
                        // and can only render what we allowed in OpacityRequest
                        panic!("Got RenderResult that is not expected - should not happen");
                    }

                    // Expected volume is updated, can we satisfy request from queue?
                    if let Some(b) = queue.back() {
                        if block_info[expected_volume].index == b.order {
                            let r = queue.pop_back().unwrap();
                            let responder = &self.renderers[r.from_id];
                            let info = &block_info[expected_volume];
                            let response =
                                self.handle_request(info, &subcanvas_opacity[..], &subcanvas_size);
                            responder.send(response).unwrap();
                        }
                    }

                    // Got all results? Convert to RGB bytes and send to master thread for output
                    let byte_canvas = self.convert_to_bytes(&subcanvas_rgb[..]);

                    subcanvas_rgb
                        .iter_mut()
                        .for_each(|v| *v = Vector3::<f32>::zeros());
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

    fn get_opacity(&self, opacity: &[f32], subframe: &PixelBox, pixels: &PixelBox) -> Vec<f32> {
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

    fn copy_opacity(&self, subcanvas_opacity: &mut [f32], frame: &PixelBox, res: &SubRenderResult) {
        let frame_width = frame.x.end - frame.x.start;
        let line_width = res.pixels.x.end - res.pixels.x.start;

        let frame_local = (
            res.pixels.x.start - frame.x.start,
            res.pixels.y.start - frame.y.start,
        );

        let mut ptr = frame_local.0 + frame_local.1 * frame_width;
        let mut res_ptr = 0;

        for _ in res.pixels.y.clone() {
            subcanvas_opacity[ptr..ptr + line_width]
                .copy_from_slice(&res.opacities[res_ptr..res_ptr + line_width]);

            ptr += frame_width;
            res_ptr += line_width;
        }
    }

    fn add_color(&self, rgb: &mut [Vector3<f32>], frame: &PixelBox, res: &SubRenderResult) {
        let frame_width = frame.x.end - frame.x.start;
        let line_width = res.pixels.x.end - res.pixels.x.start;

        let frame_local = (
            res.pixels.x.start - frame.x.start,
            res.pixels.y.start - frame.y.start,
        );

        let mut ptr = frame_local.0 + frame_local.1 * frame_width;
        let mut res_ptr = 0;

        for _ in res.pixels.y.clone() {
            rgb[ptr..ptr + line_width].copy_from_slice(&res.colors[res_ptr..res_ptr + line_width]);

            ptr += frame_width;
            res_ptr += line_width;
        }
    }

    fn convert_to_bytes(&self, subcanvas_rgb: &[Vector3<f32>]) -> Vec<u8> {
        let mut v = Vec::with_capacity(3 * subcanvas_rgb.len());
        subcanvas_rgb.iter().for_each(|rgb| {
            rgb.iter().for_each(|&val| v.push(val as u8));
        });
        v
    }

    fn handle_request(
        &self,
        info: &BlockInfo,
        opacities: &[f32],
        subcanvas_size: &PixelBox,
    ) -> ToRendererMsg {
        let box_intersection = self.area.intersection_unchecked(&info.viewport);
        let pixels = box_intersection.get_pixel_range(self.resolution);

        // Shift pixelbox to our subframe

        let opacity_data = self.get_opacity(opacities, subcanvas_size, &pixels);
        let opacity_data = OpacityData::new(self.compositor_id, pixels, opacity_data);
        ToRendererMsg::Opacity(opacity_data)
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
