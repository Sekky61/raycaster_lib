use std::sync::{Arc, RwLock};

use crossbeam_channel::{Receiver, Sender};
use nalgebra::{Vector2, Vector3};

use crate::{
    camera::{Camera, PerspectiveCamera},
    common::{PixelBox, Ray},
    volumetric::Block,
};

use super::messages::{OpacityData, RenderTask, SubRenderResult, ToCompositorMsg, ToRendererMsg};

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
}
