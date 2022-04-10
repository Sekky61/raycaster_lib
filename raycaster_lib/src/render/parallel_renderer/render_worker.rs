use std::sync::Arc;

use crossbeam::select;
use nalgebra::{vector, Vector3};
use parking_lot::RwLock;

use crate::{
    common::Ray,
    render::RenderOptions,
    volumetric::{volumes::Block, Blocked, Volume},
    PerspectiveCamera, TF,
};

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

pub struct RenderWorker<'a, BV>
where
    BV: Volume + Blocked,
{
    // todo generic blocktype
    // todo render options?
    renderer_id: usize,
    camera: Arc<RwLock<PerspectiveCamera>>,
    render_quality: bool,
    render_options: RenderOptions,
    comms: RenderWorkerComms,
    volume: &'a BV,
}

impl<'a, BV> RenderWorker<'a, BV>
where
    BV: Volume + Blocked,
{
    #[must_use]
    pub fn new(
        renderer_id: usize,
        camera: Arc<RwLock<PerspectiveCamera>>,
        render_options: RenderOptions,
        comms: RenderWorkerComms,
        volume: &'a BV,
    ) -> Self {
        Self {
            renderer_id,
            camera,
            render_quality: true,
            render_options,
            comms,
            volume,
        }
    }

    pub fn run(&mut self) {
        let mut command = None;
        loop {
            let msg = match command.take() {
                Some(cmd) => cmd,
                None => self.comms.command_rec.recv().unwrap(),
            };
            let cont = match msg {
                ToWorkerMsg::GoIdle => Run::Continue,
                ToWorkerMsg::GoLive { quality } => {
                    self.render_quality = quality;
                    Run::Render
                }
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
        let camera = self.camera.read();

        #[cfg(debug_assertions)]
        println!("Render {}: entering main loop", self.renderer_id);

        let blocks = self.volume.get_blocks();

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

            let block = &blocks[block_id as usize];

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

    fn render_block(
        &self,
        camera: &PerspectiveCamera,
        subcanvas: &mut SubCanvas,
        block: &<BV as Blocked>::BlockType,
    ) {
        // todo use renderoptions properly
        // Image size, todo move to property
        let res_f = self.render_options.resolution.map(|v| v as f32);
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
                let color = self.sample_color(block, &ray, camera, &mut opacities[ptr]);

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

    fn sample_color(
        &self,
        block: &<BV as Blocked>::BlockType,
        ray: &Ray,
        camera: &PerspectiveCamera,
        opacity: &mut f32,
    ) -> Vector3<f32> {
        let mut accum = vector![0.0, 0.0, 0.0];

        let obj_ray = block.transform_ray(ray);

        let view_dir_neg = -camera.get_dir();
        let light_dir = vector![-1.0, -1.0, -1.0].normalize(); // light direction

        let (obj_ray, t) = match obj_ray {
            Some(r) => r,
            None => return accum,
        };

        let step_size = if self.render_quality {
            // todo more render_options
            self.render_options.ray_step_quality
        } else {
            self.render_options.ray_step_fast
        };
        let max_n_of_steps = (t / step_size) as usize;

        let step = obj_ray.direction * step_size; // normalized

        let mut pos = obj_ray.origin;

        let tf = self.volume.get_tf();

        for _ in 0..max_n_of_steps {
            //let sample = self.volume.sample_at(pos);
            if *opacity > 0.99 {
                break;
            }

            let (sample, grad_samples) = block.sample_at_gradient(pos);

            pos += step;

            let color_b = tf(sample);
            if color_b.w == 0.0 {
                continue;
            }

            // Inverted, as low values indicate outside
            let grad = vector![
                sample - grad_samples.x,
                sample - grad_samples.y,
                sample - grad_samples.z
            ];

            let grad_magnitude = grad.magnitude();
            const GRAD_MAG_THRESH: f32 = 10.0; // todo tweak

            let mut sample_rgb = color_b.xyz();

            if grad_magnitude > GRAD_MAG_THRESH {
                let grad_norm = grad / grad_magnitude;
                let diffuse = f32::max(grad_norm.dot(&-light_dir), 0.00); // ambient light 0.09

                let reflect = light_dir - 2.0 * (grad_norm.dot(&light_dir)) * grad_norm;
                let r_dot_view = reflect.dot(&view_dir_neg);
                let light_intensity = 200.0;
                let specular = f32::max(0.0, r_dot_view).powf(128.0) * light_intensity;

                sample_rgb = sample_rgb * (diffuse + 0.09) + vector![specular, specular, specular];
            }

            // pseudocode from https://scholarworks.rit.edu/cgi/viewcontent.cgi?article=6466&context=theses page 55, figure 5.6
            //sum = (1 - sum.alpha) * volume.density * color + sum;

            accum += (1.0 - *opacity) * color_b.w * sample_rgb;

            *opacity += (1.0 - *opacity) * color_b.w;
        }

        accum
    }
}
