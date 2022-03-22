use std::{
    collections::VecDeque,
    sync::{Arc, RwLock},
};

use crossbeam::select;
use crossbeam_channel::{Receiver, Sender};
use nalgebra::{Vector2, Vector3};

use crate::{
    camera::{Camera, PerspectiveCamera},
    common::{PixelBox, ViewportBox},
    render::paralel_renderer::messages::{OpacityRequest, SubFrameResult},
    volumetric::Block,
};

use super::messages::{
    OpacityData, SubRenderResult, ToCompositorMsg, ToMasterMsg, ToRendererMsg, ToWorkerMsg,
};

enum Run {
    Stop,
    Continue,
    Render,
}

// todo just add reference to renderer?
pub struct CompositorWorker<'a> {
    compositor_id: usize,
    camera: Arc<RwLock<PerspectiveCamera>>,
    area: ViewportBox, // todo remove, count in pixels
    pixels: PixelBox,
    resolution: Vector2<usize>, // Resolution of full image
    // Comp x renderer comms
    renderers: [Sender<ToRendererMsg>; 4],
    receiver: Receiver<ToCompositorMsg>,
    // Comp x master comms
    result_sender: Sender<ToMasterMsg>,
    command_receiver: Receiver<ToWorkerMsg>,
    blocks: &'a [Block],
}

impl<'a> CompositorWorker<'a> {
    #[must_use]
    pub fn new(
        compositor_id: usize,
        camera: Arc<RwLock<PerspectiveCamera>>,
        area: ViewportBox,
        pixels: PixelBox,
        resolution: Vector2<usize>,
        renderers: [Sender<ToRendererMsg>; 4],
        receiver: Receiver<ToCompositorMsg>,
        result_sender: Sender<ToMasterMsg>,
        command_receiver: Receiver<ToWorkerMsg>,
        blocks: &'a [Block],
    ) -> Self {
        Self {
            compositor_id,
            camera,
            area,
            pixels,
            resolution,
            renderers,
            receiver,
            result_sender,
            command_receiver,
            blocks,
        }
    }

    pub fn run(&self) {
        let mut command = None;
        loop {
            let msg = match command.take() {
                Some(cmd) => cmd,
                None => self.command_receiver.recv().unwrap(),
            };
            let cont = match msg {
                ToWorkerMsg::GoIdle => Run::Continue,
                ToWorkerMsg::StopRendering => Run::Continue,
                ToWorkerMsg::GoLive => Run::Render,
                ToWorkerMsg::Finish => Run::Stop,
            };
            command = match cont {
                Run::Stop => break,
                Run::Continue => None,
                Run::Render => Some(self.active_state()),
            }
        }
    }

    pub fn active_state(&self) -> ToWorkerMsg {
        // Subcanvas
        let subcanvas_items = self.pixels.items();
        let mut subcanvas_rgb = vec![Vector3::<f32>::zeros(); subcanvas_items]; // todo RGB
        let mut subcanvas_opacity = vec![0.0; subcanvas_items];

        // Calculate info about blocks
        // Only subvolumes that appear in my subcanvas
        let block_info = self.get_block_info();

        #[cfg(debug_assertions)]
        println!(
            "Comp {}: list of relevant blocks: {:?}",
            self.compositor_id,
            &block_info[..]
        );

        let mut expected_volume_index = 0;
        let mut expected_volume_order = block_info[expected_volume_index].order; // pointer into block_info, todo peekable iter
        let mut queue: VecDeque<OpacityRequest> = VecDeque::new(); // todo maybe map, items can be removed... or just dont remove items

        loop {
            // Receive requests
            let request = select! {
                recv(self.receiver) -> msg => msg.unwrap(),
                recv(self.command_receiver) -> msg => return msg.unwrap(),
            };
            match request {
                ToCompositorMsg::OpacityRequest(req) => {
                    let responder = &self.renderers[req.from_id];

                    #[cfg(debug_assertions)]
                    println!(
                        "Comp {}: received request order {}",
                        self.compositor_id, req.order
                    );

                    match block_info.binary_search_by(|item| item.order.cmp(&req.order)) {
                        Ok(index) => {
                            // Block is in compositors field

                            // todo move forward, to skip binary search
                            if expected_volume_index == index {
                                // Block is up

                                let info = &block_info[index];

                                let response = self.handle_request(info, &subcanvas_opacity[..]);

                                responder.send(response).unwrap();
                                #[cfg(debug_assertions)]
                                println!(
                                    "Comp {}: responding immed. order {}",
                                    self.compositor_id, req.order
                                );
                            } else {
                                // Needs to be placed in queue
                                #[cfg(debug_assertions)]
                                println!(
                                    "Comp {}: queuing order {} because expecting order {expected_volume_order}[{expected_volume_index}]",
                                    self.compositor_id, req.order
                                );

                                let q_index = queue.binary_search_by(|re| re.order.cmp(&req.order));
                                match q_index {
                                    Ok(i) => panic!("Already in queue"),
                                    Err(ins_i) => queue.insert(ins_i, req),
                                }
                            }
                        }
                        Err(_) => {
                            // Block is not in compositors field
                            let response = ToRendererMsg::EmptyOpacity;
                            responder.send(response).unwrap();

                            #[cfg(debug_assertions)]
                            println!(
                                "Comp {}: responding empty order {}",
                                self.compositor_id, req.order
                            );
                        }
                    }
                }
                ToCompositorMsg::RenderResult(res) => {
                    // SubRenderResult buffers have the same shape as sent in their previous request

                    #[cfg(debug_assertions)]
                    println!(
                        "Comp {}: got result order {}",
                        self.compositor_id, res.order
                    );

                    if expected_volume_order == res.order {
                        // Block is expected

                        // Update opacity
                        self.copy_opacity(&mut subcanvas_opacity[..], &res);

                        // Update color
                        self.add_color(&mut subcanvas_rgb[..], &self.pixels, &res);

                        // Update next expected volume
                        expected_volume_index += 1;
                    } else {
                        // Not expected
                        // Renderer is sending only nonempty results
                        // and can only render what we allowed in OpacityRequest
                        panic!("Got RenderResult that is not expected - should not happen");
                    }

                    if expected_volume_index == block_info.len() {
                        // Got all results
                        self.send_subcanvas(&subcanvas_rgb);

                        // Reset color buffer
                        subcanvas_rgb
                            .iter_mut()
                            .for_each(|v| *v = Vector3::<f32>::zeros());

                        continue;
                    }

                    // Now we can be sure index is valid
                    expected_volume_order = block_info[expected_volume_index].order;

                    #[cfg(debug_assertions)]
                    println!(
                        "Comp {}: checking queue looking for order {expected_volume_order} - {:?}",
                        self.compositor_id, queue
                    );

                    // Expected volume is updated, can we satisfy request from queue?
                    // Note that we cannot start more than one
                    if let Ok(q_index) =
                        queue.binary_search_by(|req| req.order.cmp(&expected_volume_order))
                    {
                        let req = queue[q_index];

                        queue.remove(q_index);

                        //let r = queue.pop_front().unwrap();
                        let responder = &self.renderers[req.from_id];
                        let info = &block_info[expected_volume_index];
                        let response = self.handle_request(info, &subcanvas_opacity[..]);
                        responder.send(response).unwrap();

                        #[cfg(debug_assertions)]
                        println!(
                            "Comp {}: responding from queue {}",
                            self.compositor_id, req.order
                        );
                    } else {
                        #[cfg(debug_assertions)]
                        println!("Comp {}: nothing from queue satisfied", self.compositor_id);
                    }

                    if expected_volume_index == block_info.len() {
                        // Got all results
                        self.send_subcanvas(&subcanvas_rgb);

                        // Reset color buffer
                        subcanvas_rgb
                            .iter_mut()
                            .for_each(|v| *v = Vector3::<f32>::zeros());
                    }
                }
            }
        }
    }

    fn send_subcanvas(&self, subcanvas_rgb: &[Vector3<f32>]) {
        // Convert to RGB bytes and send to master thread for output
        let byte_canvas = convert_to_bytes(subcanvas_rgb);

        let offset = self.resolution.x * self.pixels.y.start + self.pixels.x.start;
        let width = self.pixels.x.end - self.pixels.x.start;

        // Send byte canvas to master
        let res = SubFrameResult::new(byte_canvas, offset, width);
        self.result_sender.send(ToMasterMsg::Subframe(res)).unwrap();

        #[cfg(debug_assertions)]
        println!("Comp {}: sent canvas", self.compositor_id);
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
        // todo sort once on main thread and share results?
        let mut relevant_ids = vec![];
        {
            let camera = self.camera.read().unwrap();

            for (i, block) in self.blocks.iter().enumerate() {
                let viewport = camera.project_box(block.bound_box);
                let distance = camera.box_distance(&block.bound_box);
                let info = BlockInfo::new(i, 0, distance, viewport);
                relevant_ids.push(info);
            }
        }
        relevant_ids.sort_unstable_by(|b1, b2| b1.distance.partial_cmp(&b2.distance).unwrap());

        relevant_ids = relevant_ids
            .into_iter()
            .enumerate()
            .map(|(i, mut info)| {
                info.order = i;
                info
            })
            .filter(|info| self.area.crosses(&info.viewport))
            .collect();

        relevant_ids
    }

    // Copy rectangle of opacity data
    fn get_opacity(&self, opacity: &[f32], subframe: &PixelBox) -> Vec<f32> {
        // todo check for same issues from single thread
        let pixels = &self.pixels;
        let mut v = Vec::with_capacity(subframe.items());
        let subframe_width = subframe.x.end - subframe.x.start;
        let subframe_height = subframe.y.end - subframe.y.start;
        let width = self.pixels.x.end - self.pixels.x.start;

        let subframe_offset_x = subframe.x.start - pixels.x.start;
        let subframe_offset_y = subframe.y.start - pixels.y.start;

        let mut line_start = subframe_offset_y * width + subframe_offset_x;
        for _ in 0..subframe_height {
            v.extend(&opacity[line_start..line_start + subframe_width]);
            line_start += width;
        }
        v
    }

    fn copy_opacity(&self, dest: &mut [f32], res: &SubRenderResult) {
        let dest_width = self.pixels.x.end - self.pixels.x.start;
        let line_width = res.pixels.x.end - res.pixels.x.start;

        let frame_local = (
            res.pixels.x.start - self.pixels.x.start,
            res.pixels.y.start - self.pixels.y.start,
        );

        let mut ptr = frame_local.0 + frame_local.1 * dest_width;
        let mut res_ptr = 0;

        for _ in res.pixels.y.clone() {
            dest[ptr..ptr + line_width]
                .copy_from_slice(&res.opacities[res_ptr..res_ptr + line_width]);

            ptr += dest_width;
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

    fn handle_request(&self, info: &BlockInfo, opacities: &[f32]) -> ToRendererMsg {
        let box_intersection = self.area.intersection_unchecked(&info.viewport);
        let pixels = box_intersection.get_pixel_range(self.resolution);

        // Shift pixelbox to our subframe

        let opacity_data = self.get_opacity(opacities, &pixels);
        let opacity_data = OpacityData::new(self.compositor_id, pixels, opacity_data);
        ToRendererMsg::Opacity(opacity_data)
    }
}

fn convert_to_bytes(subcanvas_rgb: &[Vector3<f32>]) -> Vec<u8> {
    let mut v = Vec::with_capacity(3 * subcanvas_rgb.len());
    subcanvas_rgb.iter().for_each(|rgb| {
        rgb.iter().for_each(|&val| v.push(val as u8));
    });
    v
}

pub struct BlockInfo {
    index: usize,
    order: usize,
    distance: f32,
    viewport: ViewportBox,
}

impl BlockInfo {
    pub fn new(index: usize, order: usize, distance: f32, viewport: ViewportBox) -> Self {
        Self {
            index,
            order,
            distance,
            viewport,
        }
    }
}

impl std::fmt::Debug for BlockInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("B ord {}", self.order))
    }
}

#[cfg(test)]
mod test {

    use nalgebra::{point, vector};

    use super::*;
    use crate::{common::Ray, test_helpers::*};

    #[test]
    fn canvas_to_bytes() {
        // Test size

        let canvas = &[vector![0.0, 0.0, 0.0]; 4][..];

        let bytes_vec = convert_to_bytes(canvas);

        assert_eq!(bytes_vec.len(), 12);
        for byte in bytes_vec {
            assert_eq!(byte, 0);
        }

        // Test values

        let canvas = &[vector![1.4, 20.4, 6.95], vector![0.0, 0.1, 254.1]][..];

        let bytes_vec = convert_to_bytes(canvas);

        assert_eq!(bytes_vec.len(), 6);
        assert_eq!(&bytes_vec[..], &[1, 20, 6, 0, 0, 254]);
    }
}
