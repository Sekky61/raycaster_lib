use std::sync::{Arc, RwLock};

use crossbeam_channel::{Receiver, Sender};
use nalgebra::{Vector2, Vector3};

use crate::{
    camera::{Camera, PerspectiveCamera},
    common::{PixelBox, Ray},
    volumetric::Block,
};

use super::{
    compositor_worker::BlockInfo,
    messages::{
        OpacityData, OpacityRequest, RenderTask, SubRenderResult, ToCompositorMsg, ToRendererMsg,
    },
};

pub struct RenderWorker<'a> {
    renderer_id: usize,
    camera: Arc<RwLock<PerspectiveCamera>>,
    resolution: Vector2<usize>,
    compositors: [Sender<ToCompositorMsg>; 4],
    receiver: Receiver<ToRendererMsg>,
    task_receiver: Receiver<RenderTask>,
    blocks: &'a [Block],
}

impl<'a> RenderWorker<'a> {
    pub fn new(
        renderer_id: usize,
        camera: Arc<RwLock<PerspectiveCamera>>,
        resolution: Vector2<usize>,
        compositors: [Sender<ToCompositorMsg>; 4],
        receiver: Receiver<ToRendererMsg>,
        task_receiver: Receiver<RenderTask>,
        blocks: &'a [Block],
    ) -> Self {
        Self {
            renderer_id,
            camera,
            resolution,
            compositors,
            receiver,
            task_receiver,
            blocks,
        }
    }

    pub fn run(&self) {
        let camera = self
            .camera
            .read()
            .expect("Cannot acquire read lock to camera");

        loop {
            // Wait for task from master thread or finish call
            let task = self.task_receiver.recv().unwrap();
            let block_order = task.block_order;

            // Ask for all opacity data
            for comp in self.compositors.iter() {
                let op_req = OpacityRequest::new(self.renderer_id, block_order);
                comp.send(ToCompositorMsg::OpacityRequest(op_req)).unwrap();
            }

            // Wait for all opacity data
            let opacities = {
                for _ in 0..self.compositors.len() {
                    let msg = self.receiver.recv().unwrap();
                    match msg {
                        ToRendererMsg::Opacity(d) => todo!(),
                        ToRendererMsg::EmptyOpacity => todo!(),
                    }
                }
            };

            // Get data from compositers

            // Render task

            // give data to compositers
        }
    }

    fn render_block(
        &self,
        camera: &PerspectiveCamera,
        data: &mut OpacityData,
        block: &Block,
    ) -> Vec<Vector3<f32>> {
        // get viewport box
        let vpb = camera.project_box(block.bound_box);

        // Image size, todo move to property
        let res_f = self.resolution.map(|v| v as f32);
        let step_f = res_f.map(|v| 1.0 / v);

        let PixelBox {
            x: x_range,
            y: y_range,
        } = vpb.get_pixel_range(self.resolution);

        // Request opacity data
        let mut colors = vec![];
        let mut opacities = vec![];

        for y in y_range {
            let y_norm = y as f32 * step_f.y;
            for x in x_range.clone() {
                // todo clone here -- maybe use own impl
                let pixel_coord = (x as f32 * step_f.x, y_norm);
                let ray = camera.get_ray(pixel_coord);

                let (color, opacity) = self.sample_color(block, ray);

                colors.push(color);
                opacities.push(opacity);

                // Add to opacity buffer
            }
        }
        let width = x_range.end - x_range.start;

        colors
    }

    fn sample_color(&self, block: &Block, ray: Ray) -> (Vector3<f32>, f32) {
        todo!()
    }

    // Return collection of blocks in the subframe
    // Collection is sorted by distance (asc.)
    fn get_block_info(&self) -> Vec<(usize, f32)> {
        let mut relevant_ids = vec![];
        {
            let camera = self.camera.read().unwrap();

            for (i, block) in self.blocks.iter().enumerate() {
                let distance = camera.box_distance(&block.bound_box);
                relevant_ids.push((i, distance));
            }
        }
        relevant_ids.sort_unstable_by(|b1, b2| b1.1.partial_cmp(&b2.1).unwrap());

        relevant_ids
    }
}
