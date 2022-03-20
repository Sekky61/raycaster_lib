use std::sync::{Arc, RwLock};

use crossbeam_channel::{Receiver, Sender};
use nalgebra::{point, vector, Matrix4, Vector2, Vector3};

use crate::{
    camera::{Camera, PerspectiveCamera},
    common::Ray,
    volumetric::Block,
};

use super::messages::{
    OpacityRequest, RenderTask, SubRenderResult, ToCompositorMsg, ToRendererMsg,
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

        let ordered_ids = self.get_block_info();

        loop {
            // Wait for task from master thread or finish call
            let task = self.task_receiver.recv().unwrap();
            let block_order = task.block_order;
            let (block_id, _) = ordered_ids[block_order];

            #[cfg(debug_assertions)]
            println!(
                "Render {}: got task order {block_order} block id {block_id}",
                self.renderer_id
            );

            // Ask for all opacity data
            for comp in self.compositors.iter() {
                let op_req = OpacityRequest::new(self.renderer_id, block_order);
                comp.send(ToCompositorMsg::OpacityRequest(op_req)).unwrap();
            }
            #[cfg(debug_assertions)]
            println!("Render {}: requested {block_order}", self.renderer_id);

            // Wait for all opacity data from compositors
            let mut color_opacity = {
                let mut ship_back: [Option<SubRenderResult>; 4] = Default::default(); // todo const generics
                for _ in 0..self.compositors.len() {
                    let msg = self.receiver.recv().unwrap();
                    if let ToRendererMsg::Opacity(d) = msg {
                        let i = d.from_compositor;
                        let capacity = d.pixels.items();
                        let res = SubRenderResult::with_capacity(
                            block_id,
                            d.pixels,
                            capacity,
                            d.opacities,
                        );
                        ship_back[i] = Some(res);
                    }
                }
                ship_back
            };

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
            for (i, res_opt) in color_opacity.into_iter().enumerate() {
                if let Some(res) = res_opt {
                    let msg = ToCompositorMsg::RenderResult(res);
                    self.compositors[i].send(msg).unwrap();
                }
            }

            #[cfg(debug_assertions)]
            println!("Render {}: sent back {block_order}", self.renderer_id);
        }
    }

    fn render_block(
        &self,
        camera: &PerspectiveCamera,
        data: &mut [Option<SubRenderResult>; 4],
        block: &Block,
    ) {
        // Image size, todo move to property
        let res_f = self.resolution.map(|v| v as f32);
        let step_f = res_f.map(|v| 1.0 / v);

        for opacity_data in data.iter_mut().flatten() {
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

    fn sample_color(&self, block: &Block, ray: &Ray, opacity: &mut f32) -> Vector3<f32> {
        let mut accum = vector![0.0, 0.0, 0.0];

        let (t0, t1) = match block.bound_box.intersect(ray) {
            Some(t) => t,
            None => return accum,
        };
        let t = t1 - t0;

        let scale_inv = vector![1.0, 1.0, 1.0]; // todo scale
        let lower_vec = block.bound_box.lower - point![0.0, 0.0, 0.0];

        let transform = Matrix4::identity()
            .append_translation(&-lower_vec)
            .append_nonuniform_scaling(&scale_inv);

        let obj_origin = ray.point_from_t(t0);

        let origin = transform.transform_point(&obj_origin);

        let direction = ray.direction.component_mul(&scale_inv);
        let direction = direction.normalize();

        let obj_ray = Ray::from_3(origin, direction);

        let begin = obj_ray.origin;
        let direction = ray.get_direction();

        let step_size = 1.0;
        let max_n_of_steps = (t / step_size) as usize;

        let step = direction * step_size; // normalized

        let mut pos = begin;

        let tf = |s: f32| vector![s, s, s, 0.1];

        for _ in 0..max_n_of_steps {
            //let sample = self.volume.sample_at(pos);
            if *opacity > 0.99 {
                break;
            }

            let sample = block.sample_at(pos);

            let color_b = tf(sample);

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
