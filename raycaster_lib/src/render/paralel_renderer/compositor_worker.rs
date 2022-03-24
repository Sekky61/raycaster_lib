use std::{
    collections::VecDeque,
    sync::{Arc, RwLock},
};

use crossbeam::select;
use nalgebra::{Vector2, Vector3};

use crate::{
    camera::{Camera, PerspectiveCamera},
    common::PixelBox,
    render::paralel_renderer::messages::{OpacityRequest, SubFrameResult},
    volumetric::Block,
};

use super::{
    communication::CompWorkerComms,
    master_thread::PAR_SIDE,
    messages::{
        OpacityData, SubRenderResult, ToCompositorMsg, ToMasterMsg, ToRendererMsg, ToWorkerMsg,
    },
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
    pixels: PixelBox,
    resolution: Vector2<usize>, // Resolution of full image
    comms: CompWorkerComms<4>,  // todo generic
    blocks: &'a [Block<PAR_SIDE>],
}

impl<'a> CompositorWorker<'a> {
    #[must_use]
    pub fn new(
        compositor_id: usize,
        camera: Arc<RwLock<PerspectiveCamera>>,
        pixels: PixelBox,
        resolution: Vector2<usize>,
        comms: CompWorkerComms<4>,
        blocks: &'a [Block<PAR_SIDE>],
    ) -> Self {
        Self {
            compositor_id,
            camera,
            pixels,
            resolution,
            comms,
            blocks,
        }
    }

    pub fn run(&self) {
        let mut command = None;
        loop {
            let msg = match command.take() {
                Some(cmd) => cmd,
                None => self.comms.command_receiver.recv().unwrap(),
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

        let mut expected_iter = block_info.iter();

        let mut expected_volume = expected_iter.next();

        //let mut expected_volume_index = 0; // todo iterator wrapper
        //let mut expected_volume_order = block_info[expected_volume_index].order; // pointer into block_info, todo peekable iter
        let mut queue: VecDeque<OpacityRequest> = VecDeque::new(); // todo maybe map, items can be removed... or just dont remove items

        loop {
            // Receive requests
            let request = select! {
                recv(self.comms.receiver) -> msg => msg.unwrap(),
                recv(self.comms.command_receiver) -> msg => return msg.unwrap(),
            };
            match request {
                ToCompositorMsg::OpacityRequest(req) => {
                    let responder = &self.comms.renderers[req.from_id];

                    #[cfg(debug_assertions)]
                    println!(
                        "Comp {}: received request order {}",
                        self.compositor_id, req.order
                    );

                    let expected_info = match expected_volume {
                        Some(info) => info,
                        None => {
                            // All relevant blocks handled, request must not be relevant to me
                            let response = ToRendererMsg::EmptyOpacity;
                            responder.send(response).unwrap();

                            #[cfg(debug_assertions)]
                            println!(
                                "Comp {}: responding default empty order {}",
                                self.compositor_id, req.order
                            );
                            continue;
                        }
                    };

                    if req.order == expected_info.order {
                        // Block is expected, handle it

                        let response = self.handle_request(expected_info, &subcanvas_opacity[..]);

                        responder.send(response).unwrap();
                        #[cfg(debug_assertions)]
                        println!(
                            "Comp {}: responding immed. order {}",
                            self.compositor_id, req.order
                        );
                        continue;
                    }

                    // Block was not expected.
                    // Is it in my subframe?
                    match block_info.binary_search_by(|item| item.order.cmp(&req.order)) {
                        Ok(_) => {
                            // Block is in compositors subframe
                            // Needs to be placed in queue

                            let q_index = queue.binary_search_by(|re| re.order.cmp(&req.order));
                            match q_index {
                                Ok(_) => panic!("Already in queue"),
                                Err(ins_i) => {
                                    queue.insert(ins_i, req);
                                    #[cfg(debug_assertions)]
                                    println!(
                                        "Comp {}: queuing order {} because expecting order {}",
                                        self.compositor_id, req.order, expected_info.order
                                    );
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

                    let expected_info = match expected_volume {
                        Some(i) => i,
                        None => panic!("Got result, expected nothing"),
                    };

                    if expected_info.order != res.order {
                        // Not expected
                        // Renderer is sending only nonempty results
                        // and can only render what we allowed in OpacityRequest
                        panic!("Got RenderResult that is not expected - should not happen");
                    }

                    // Block is expected

                    // Update opacity
                    self.copy_opacity(&mut subcanvas_opacity[..], &res);

                    // Update color
                    self.add_color(&mut subcanvas_rgb[..], &res);

                    // Update next expected volume
                    expected_volume = expected_iter.next();

                    match expected_volume {
                        Some(block_info) => {
                            #[cfg(debug_assertions)]
                            println!(
                                "Comp {}: checking queue looking for order {} - {:?}",
                                self.compositor_id, block_info.order, queue
                            );

                            // New expected volume is updated, can we satisfy request from queue?
                            // Note that we cannot start more than one
                            // TODO queue is (probably) sorted, so just look at top
                            if let Ok(q_index) =
                                queue.binary_search_by(|req| req.order.cmp(&block_info.order))
                            {
                                let req = queue[q_index];

                                queue.remove(q_index);

                                let responder = &self.comms.renderers[req.from_id];
                                let response =
                                    self.handle_request(block_info, &subcanvas_opacity[..]);
                                responder.send(response).unwrap();

                                #[cfg(debug_assertions)]
                                println!(
                                    "Comp {}: responding from queue order {}",
                                    self.compositor_id, req.order
                                );
                            } else {
                                #[cfg(debug_assertions)]
                                println!(
                                    "Comp {}: nothing from queue satisfied",
                                    self.compositor_id
                                );
                            }
                        }
                        None => {
                            // Got all results
                            // Send subcanvas to master
                            self.send_subcanvas(&subcanvas_rgb);

                            // Reset color buffer
                            self.reset_frame_buffers(&mut subcanvas_rgb, &mut subcanvas_opacity);

                            continue;
                        }
                    }
                }
            }
        }
    }

    fn send_subcanvas(&self, subcanvas_rgb: &[Vector3<f32>]) {
        // Convert to RGB bytes and send to master thread for output
        let byte_canvas = convert_to_bytes(subcanvas_rgb);

        // Send byte canvas to master
        let res = SubFrameResult::new(self.compositor_id, byte_canvas);
        self.comms
            .result_sender
            .send(ToMasterMsg::Subframe(res))
            .unwrap();

        #[cfg(debug_assertions)]
        println!("Comp {}: sent canvas", self.compositor_id);
    }

    // Return collection of blocks in the subframe
    // Collection is sorted by distance (asc.)
    fn get_block_info(&self) -> Vec<BlockInfo> {
        // todo sort once on main thread and share results?
        let mut distances = Vec::with_capacity(self.blocks.len());

        let camera = self.camera.read().unwrap(); // Lock guard

        for (i, block) in self.blocks.iter().enumerate() {
            let distance = camera.box_distance(&block.bound_box);
            distances.push((i, distance));
        }

        distances.sort_unstable_by(|b1, b2| b1.1.partial_cmp(&b2.1).unwrap());

        let mut relevant_blocks = Vec::with_capacity(self.blocks.len());

        for (order, (id, _)) in distances.into_iter().enumerate() {
            let block = &self.blocks[id];
            let viewport = camera.project_box(block.bound_box);
            let pixels = viewport.get_pixel_range(self.resolution);
            if let Some(inter) = pixels.intersection(&self.pixels) {
                // TODO is crosses check faster? probably
                let info = BlockInfo::new(order, inter);
                relevant_blocks.push(info);
            }
        }

        relevant_blocks
    }

    // Copy rectangle of opacity data
    // parameter subframe is in global frame coords, needs to be shifted
    fn get_opacity(&self, opacity: &[f32], subframe: &PixelBox) -> Vec<f32> {
        // todo check for same issues from single thread
        let mut v = Vec::with_capacity(subframe.items());
        let subframe_width = subframe.width();
        let subframe_height = subframe.height();
        let width = self.pixels.width();

        let subframe_offset_x = subframe.x.start - self.pixels.x.start;
        let subframe_offset_y = subframe.y.start - self.pixels.y.start;

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

    fn add_color(&self, rgb: &mut [Vector3<f32>], res: &SubRenderResult) {
        let frame_width = self.pixels.width();
        let line_width = res.pixels.width();

        let offset = self.pixels.offset_in_unchecked(&res.pixels);

        let mut ptr = offset;
        let mut res_ptr = 0;

        for _ in res.pixels.y.clone() {
            // Colors should be multiplied by opacity by renderers
            for pix in 0..line_width {
                rgb[ptr + pix] += res.colors[res_ptr + pix];
            }

            ptr += frame_width;
            res_ptr += line_width;
        }
    }

    fn handle_request(&self, info: &BlockInfo, opacities: &[f32]) -> ToRendererMsg {
        let opacity_data = self.get_opacity(opacities, &info.pixels);
        let opacity_data = OpacityData::new(self.compositor_id, info.pixels.clone(), opacity_data);
        ToRendererMsg::Opacity(opacity_data)
    }

    fn reset_frame_buffers(
        &self,
        subcanvas_rgb: &mut [Vector3<f32>],
        subcanvas_opacities: &mut [f32],
    ) {
        for v in subcanvas_rgb {
            *v = Vector3::<f32>::zeros();
        }

        for v in subcanvas_opacities {
            *v = 0.0;
        }
    }
}

fn convert_to_bytes(subcanvas_rgb: &[Vector3<f32>]) -> Vec<u8> {
    let mut v = Vec::with_capacity(3 * subcanvas_rgb.len());
    subcanvas_rgb.iter().for_each(|rgb| {
        rgb.iter().for_each(|&val| v.push(val as u8));
    });
    v
}

// Compositor needs to know about order of blocks and viewport box of a block
pub struct BlockInfo {
    order: usize,
    pixels: PixelBox,
}

impl BlockInfo {
    #[must_use]
    pub fn new(order: usize, pixels: PixelBox) -> Self {
        Self { order, pixels }
    }
}

impl std::fmt::Debug for BlockInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("B ord {}", self.order))
    }
}

#[cfg(test)]
mod test {

    use nalgebra::vector;

    use super::*;

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
