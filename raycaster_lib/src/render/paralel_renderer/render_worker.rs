use std::sync::{Arc, RwLock};

use arrayvec::ArrayVec;
use crossbeam::select;
use nalgebra::{point, vector, Matrix4, Vector2, Vector3};

use crate::{
    camera::{Camera, PerspectiveCamera},
    common::Ray,
    volumetric::{Block, TF},
};

use super::{
    communication::RenderWorkerComms,
    master_thread::PAR_SIDE,
    messages::{OpacityRequest, SubRenderResult, ToCompositorMsg, ToRendererMsg, ToWorkerMsg},
};

enum Run {
    Stop,
    Continue,
    Render,
}

pub struct RenderWorker<'a> {
    renderer_id: usize,
    camera: Arc<RwLock<PerspectiveCamera>>,
    tf: TF,
    resolution: Vector2<usize>,
    comms: RenderWorkerComms<4>, // todo generic
    blocks: &'a [Block<PAR_SIDE>],
}

impl<'a> RenderWorker<'a> {
    #[must_use]
    pub fn new(
        renderer_id: usize,
        camera: Arc<RwLock<PerspectiveCamera>>,
        tf: TF,
        resolution: Vector2<usize>,
        comms: RenderWorkerComms<4>,
        blocks: &'a [Block<PAR_SIDE>],
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

    fn active_state(&self) -> ToWorkerMsg {
        let camera = self
            .camera
            .read()
            .expect("Cannot acquire read lock to camera");

        let ordered_ids = self.get_block_info();

        loop {
            // Wait for task from master thread or finish call
            let task = select! {
                recv(self.comms.task_receiver) -> msg => msg.unwrap(),
                recv(self.comms.command_receiver) -> msg => return msg.unwrap(),
            };

            let block_order = task.block_order;
            let (block_id, _) = ordered_ids[block_order];

            #[cfg(debug_assertions)]
            println!(
                "Render {}: got task order {block_order} block id {block_id}",
                self.renderer_id
            );

            // Ask for all opacity data
            // TODO it could be known in which subframes the block is (part of task), eliminating some searches in expected_volumes
            for comp in self.comms.compositors.iter() {
                let op_req = OpacityRequest::new(self.renderer_id, block_order);
                comp.send(ToCompositorMsg::OpacityRequest(op_req)).unwrap();
            }
            #[cfg(debug_assertions)]
            println!("Render {}: requested order {block_order}", self.renderer_id);

            // Wait for all opacity data from compositors
            let mut color_opacity = ArrayVec::<SubRenderResult, 4>::new(); // todo const generics
            for _ in 0..self.comms.compositors.len() {
                let msg = self.comms.receiver.recv().unwrap();
                if let ToRendererMsg::Opacity(d) = msg {
                    let comp_id = d.from_compositor;
                    let res = SubRenderResult::new(comp_id, block_order, d.pixels, d.opacities);
                    color_opacity.push(res);
                }
            }

            #[cfg(debug_assertions)]
            println!(
                "Render {}: received opacities {block_order}",
                self.renderer_id
            );

            let block = &self.blocks[block_id];

            #[cfg(debug_assertions)]
            println!(
                "Render {}: rendering block order {block_order} (id {block_id})",
                self.renderer_id
            );

            // Render task
            self.render_block(&camera, &mut color_opacity, block);
            // Opacities has been mutated

            #[cfg(debug_assertions)]
            println!("Render {}: rendered {block_order}", self.renderer_id);

            // give data to compositers
            color_opacity.into_iter().for_each(|sub| {
                let id = sub.recipient_id;
                let msg = ToCompositorMsg::RenderResult(sub);
                self.comms.compositors[id].send(msg).unwrap();
            });

            #[cfg(debug_assertions)]
            println!("Render {}: sent back {block_order}", self.renderer_id);
        }
    }

    fn render_block(
        &self,
        camera: &PerspectiveCamera,
        data: &mut [SubRenderResult],
        block: &Block<PAR_SIDE>,
    ) {
        // Image size, todo move to property
        let res_f = self.resolution.map(|v| v as f32);
        let step_f = res_f.map(|v| 1.0 / v);

        for opacity_data in data {
            // todo waiting for opacities can be done here, render and send back immediately
            // flatten skips Nones
            let opacities = &mut opacity_data.opacities[..];

            let x_range = opacity_data.pixels.x.clone();
            let y_range = opacity_data.pixels.y.clone();

            let color_buf = &mut opacity_data.colors;
            let mut ptr = 0;

            for y in y_range {
                let y_norm = 1.0 - (y as f32 * step_f.y);
                for x in x_range.clone() {
                    // todo clone here -- maybe use own impl
                    let pixel_coord = (x as f32 * step_f.x, y_norm);
                    let ray = camera.get_ray(pixel_coord);

                    // Adds to opacity buffer
                    let color = self.sample_color(block, &ray, &mut opacities[ptr]);

                    // todo multiply color with opacity

                    color_buf.push(color);

                    ptr += 1;
                }
            }
        }
    }

    fn sample_color(&self, block: &Block<PAR_SIDE>, ray: &Ray, opacity: &mut f32) -> Vector3<f32> {
        let mut accum = vector![0.0, 0.0, 0.0];

        let obj_ray = block.transform_ray(ray);

        let (obj_ray, t) = match obj_ray {
            Some(r) => r,
            None => return accum,
        };

        let step_size = 1.0;
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

            *opacity += (1.0 - *opacity) * color_b.w;

            accum += (1.0 - *opacity) * color_b.xyz();
        }

        accum
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
