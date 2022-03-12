use std::sync::{Arc, RwLock};

use crossbeam_channel::{Receiver, Sender};
use nalgebra::Vector3;

use crate::{
    camera::{Camera, PerspectiveCamera},
    common::Ray,
    volumetric::Block,
};

use super::messages::{RenderTask, SubRenderResult, ToCompositorMsg, ToRendererMsg};

pub struct RenderWorker<'a> {
    renderer_id: usize,
    camera: Arc<RwLock<PerspectiveCamera>>,
    resolution: (usize, usize),
    compositors: [Sender<ToCompositorMsg>; 4],
    receiver: Receiver<ToRendererMsg>,
    task_receiver: Receiver<RenderTask>,
    blocks: &'a [Block],
}

impl<'a> RenderWorker<'a> {
    pub fn new(
        renderer_id: usize,
        camera: Arc<RwLock<PerspectiveCamera>>,
        resolution: (usize, usize),
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

            // Get data from compositers

            // Render task

            // give data to compositers
        }
    }

    fn render_block(&self, camera: &PerspectiveCamera, block: &Block) -> SubRenderResult {
        // get viewport box
        let vpb = camera.project_box(block.bound_box);

        // Image size, todo move to property
        let (img_w, img_h) = self.resolution;
        let (image_width, image_height) = (img_w as f32, img_h as f32);
        let step_x = 1.0 / image_width;
        let step_y = 1.0 / image_height;

        let (x_range, y_range) = vpb.get_pixel_range(self.resolution);

        // Request opacity data
        let mut colors = vec![];
        let mut opacities = vec![];

        for y in y_range {
            let y_norm = y as f32 * step_y;
            for x in x_range.clone() {
                // todo clone here -- maybe use own impl
                let pixel_coord = (x as f32 * step_x, y_norm);
                let ray = camera.get_ray(pixel_coord);

                let (color, opacity) = self.sample_color(block, ray);

                colors.push(color);
                opacities.push(opacity);

                // Add to opacity buffer
            }
        }
        let width = x_range.end - x_range.start;

        SubRenderResult::new(width, colors, opacities)
    }

    fn sample_color(&self, block: &Block, ray: Ray) -> (Vector3<f32>, f32) {
        todo!()
    }
}
