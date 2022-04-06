use std::sync::{Arc, RwLock};

use crossbeam::select;
use nalgebra::{vector, Vector2, Vector3};

use crate::{common::Ray, volumetric::Block, PerspectiveCamera, TF};

use super::{
    communication::RenderWorkerComms,
    composition::SubCanvas,
    messages::{SubRenderResult, ToWorkerMsg},
};

enum Run {
    Stop,
    Continue,
    Render,
}

pub struct RenderWorker<'a> {
    // todo render options?
    renderer_id: usize,
    camera: Arc<RwLock<PerspectiveCamera>>,
    tf: TF,
    resolution: Vector2<usize>,
    comms: RenderWorkerComms,
    blocks: &'a [Block],
}

impl<'a> RenderWorker<'a> {
    #[must_use]
    pub fn new(
        renderer_id: usize,
        camera: Arc<RwLock<PerspectiveCamera>>,
        tf: TF,
        resolution: Vector2<usize>,
        comms: RenderWorkerComms,
        blocks: &'a [Block],
    ) -> Self {
        Self {
            renderer_id,
            camera,
            tf,
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
                None => self.comms.command_rec.recv().unwrap(),
            };
            let cont = match msg {
                ToWorkerMsg::GoIdle => Run::Continue,
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

    fn active_state(&self) -> ToWorkerMsg {
        let camera = self
            .camera
            .read()
            .expect("Cannot acquire read lock to camera");

        #[cfg(debug_assertions)]
        println!("Render {}: entering main loop", self.renderer_id);

        loop {
            // Wait for task from master thread or finish call
            let task = select! {
                recv(self.comms.task_rec) -> msg => msg.unwrap(),
                recv(self.comms.command_rec) -> msg => return msg.unwrap(),
            };

            let block_id = task.block_id;

            #[cfg(debug_assertions)]
            println!("Render {}: got task block id {block_id}", self.renderer_id);

            // Safety: ref is unique
            let subcanvas = unsafe { task.subcanvas.as_mut().unwrap() };

            let block = &self.blocks[block_id];

            // Render task
            self.render_block(&camera, subcanvas, block);
            // Opacities have been mutated

            #[cfg(debug_assertions)]
            println!("Render {}: rendered {block_id}", self.renderer_id);
            let subrender_res = SubRenderResult::new(task.tile_id);
            self.comms.result_sen.send(subrender_res).unwrap();

            #[cfg(debug_assertions)]
            println!("Render {}: sent back block {block_id}", self.renderer_id);
        }
    }

    fn render_block(&self, camera: &PerspectiveCamera, subcanvas: &mut SubCanvas, block: &Block) {
        // Image size, todo move to property
        let res_f = self.resolution.map(|v| v as f32);
        let step_f = res_f.map(|v| 1.0 / v);

        // todo waiting for opacities can be done here, render and send back immediately
        // flatten skips Nones
        let opacities = &mut subcanvas.opacities[..];

        let x_range = subcanvas.pixels.x.clone();
        let y_range = subcanvas.pixels.y.clone();

        let color_buf = &mut subcanvas.colors[..];
        let mut ptr = 0;

        for y in y_range {
            let y_norm = y as f32 * step_f.y;
            for x in x_range.clone() {
                // todo clone here -- maybe use own impl
                let pixel_coord = (x as f32 * step_f.x, y_norm);
                let ray = camera.get_ray(pixel_coord);

                // Early opacity check
                if opacities[ptr] > 0.99 {
                    ptr += 1;
                    continue;
                }

                // Adds to opacity buffer
                let color = self.sample_color(block, &ray, &mut opacities[ptr]);

                // TODO multiply color with opacity ??
                // TODO results seem ok

                // if x == x_range.start
                //     || x == x_range.end - 1
                //     || y == y_range.start
                //     || y == y_range.end - 1
                // {
                //     color_buf[ptr] = vector![255.0, 255.0, 255.0];
                //     opacities[ptr] = 1.0;
                // }

                color_buf[ptr] += color;

                ptr += 1;
            }
        }
    }

    fn sample_color(&self, block: &Block, ray: &Ray, opacity: &mut f32) -> Vector3<f32> {
        let mut accum = vector![0.0, 0.0, 0.0];

        let obj_ray = block.transform_ray(ray);

        let (obj_ray, t) = match obj_ray {
            Some(r) => r,
            None => return accum,
        };

        let step_size = 0.5;
        let max_n_of_steps = (t / step_size) as usize;

        let step = obj_ray.direction * step_size; // normalized

        let mut pos = obj_ray.origin;

        for _ in 0..max_n_of_steps {
            //let sample = self.volume.sample_at(pos);
            if *opacity > 0.99 {
                break;
            }

            let sample = block.sample_at(pos);

            let color_b = (self.tf)(sample);

            pos += step;

            if color_b.w == 0.0 {
                continue;
            }

            // pseudocode from https://scholarworks.rit.edu/cgi/viewcontent.cgi?article=6466&context=theses page 55, figure 5.6
            //sum = (1 - sum.alpha) * volume.density * color + sum;

            accum += (1.0 - *opacity) * color_b.w * color_b.xyz();

            *opacity += (1.0 - *opacity) * color_b.w;
        }

        accum
    }
}
